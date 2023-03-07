from collections import namedtuple
from typing import Any

import equalto
from django.test import RequestFactory, TestCase
from django.utils.http import urlencode

from serverless.log import info
from serverless.models import License, LicenseDomain, Workbook
from serverless.schema import schema
from serverless.util import is_license_key_valid_for_host
from serverless.views import activate_license_key, send_license_key


def graphql_query(
    query: str,
    origin: str,
    license_key: str | None = None,
    variables: dict[str, Any] | None = None,
    suppress_errors: bool = False,
) -> dict[str, Any]:
    info("graphql_query(): query=%s" % query)
    context = namedtuple("context", ["META"])
    context.META = {"HTTP_ORIGIN": origin}
    if license_key is not None:
        context.META["HTTP_AUTHORIZATION"] = "Bearer %s" % str(license_key)

    info("graphql_query(): context.META=%s" % context.META)
    graphql_results = schema.execute(query, context_value=context, variable_values=variables)
    if not suppress_errors and graphql_results.errors:
        raise graphql_results.errors[0]
    return {"data": graphql_results.data}


def _create_workbook(license: License, workbook_data: dict[str, Any] | None = None) -> Workbook:
    data = graphql_query(
        """
        mutation {
            create_workbook: createWorkbook {
                workbook{id}
            }
        }
        """,
        "example.com",
        license.key,
    )
    workbook = Workbook.objects.get(id=data["data"]["create_workbook"]["workbook"]["id"])

    if workbook_data is not None:
        _set_workbook_data(workbook, workbook_data)

    return workbook


def _set_workbook_data(workbook: Workbook, workbook_data: dict[str, Any]) -> None:
    wb = equalto.new()
    for _ in range(len(workbook_data) - 1):
        wb.sheets.add()
    for sheet_index, (sheet_name, sheet_data) in enumerate(workbook_data.items()):
        sheet = wb.sheets[sheet_index]
        sheet.name = sheet_name
        for cell_ref, user_input in sheet_data.items():
            sheet[cell_ref].set_user_input(user_input)

    workbook.workbook_json = wb.json
    workbook.save()


class SimpleTest(TestCase):
    def setUp(self) -> None:
        # Every test needs access to the request factory.
        self.factory = RequestFactory()

    def test_send_license_key_invalid(self) -> None:
        # email & domains missing
        request = self.factory.post("/send-license-key")
        response = send_license_key(request)
        self.assertEqual(response.status_code, 400)

        # email missing
        request = self.factory.post("/send-license-key?domains=example.com")
        response = send_license_key(request)
        self.assertEqual(response.status_code, 400)

        # domain missing
        request = self.factory.post("/send-license-key?%s" % urlencode({"email": "joe@example.com"}))
        response = send_license_key(request)
        self.assertEqual(response.status_code, 400)

        # domain missing
        request = self.factory.post("/send-license-key?%s" % urlencode({"email": "joe@example.com", "domains": ""}))
        response = send_license_key(request)
        self.assertEqual(response.status_code, 400)

    def test_send_license_key(self) -> None:
        self.assertEqual(License.objects.count(), 0)

        request = self.factory.post(
            "/send-license-key?%s"
            % urlencode(
                {
                    "email": "joe@example.com",
                    "domains": "example.com,example2.com,*.example3.com",
                },
            ),
        )
        response = send_license_key(request, _send_email=False)
        self.assertEqual(response.status_code, 200)
        self.assertEqual(License.objects.count(), 1)
        self.assertListEqual(
            list(License.objects.values_list("email", "email_verified")),
            [("joe@example.com", False)],
        )
        license = License.objects.get()
        self.assertCountEqual(
            LicenseDomain.objects.values_list("license", "domain"),
            [
                (license.id, "example.com"),
                (license.id, "example2.com"),
                (license.id, "*.example3.com"),
            ],
        )

        # license email not verified, so license not activate
        self.assertFalse(is_license_key_valid_for_host(license.key, "example.com:443"))
        self.assertFalse(is_license_key_valid_for_host(license.key, "example2.com:443"))
        self.assertFalse(is_license_key_valid_for_host(license.key, "sub.example3.com:443"))
        # these aren't valid, regardless of license activation
        self.assertFalse(is_license_key_valid_for_host(license.key, "other.com:443"))
        self.assertFalse(is_license_key_valid_for_host(license.key, "sub.example.com:443"))

        # verify email address, activating license
        request = self.factory.get("/activate-license-key/%s/" % license.id)
        response = activate_license_key(request, license.id)
        self.assertEqual(response.status_code, 200)
        license.refresh_from_db()
        self.assertTrue(license.email_verified)

        # license email verified, so license not activate
        self.assertTrue(is_license_key_valid_for_host(license.key, "example.com:443"))
        self.assertTrue(is_license_key_valid_for_host(license.key, "example2.com:443"))
        self.assertTrue(is_license_key_valid_for_host(license.key, "sub.example3.com:443"))
        # these aren't valid, regardless of license activation
        self.assertFalse(is_license_key_valid_for_host(license.key, "other.com:443"))
        self.assertFalse(is_license_key_valid_for_host(license.key, "sub.example.com:443"))

    def _create_verified_license(
        self,
        email: str = "joe@example.com",
        domains: str = "example.com,example2.com,*.example3.com",
    ) -> License:
        before_license_ids = list(License.objects.values_list("id", flat=True))
        request = self.factory.post("/send-license-key?%s" % urlencode({"email": email, "domains": domains}))
        send_license_key(request, _send_email=False)
        after_license_ids = License.objects.values_list("id", flat=True)
        new_license_ids = list(set(after_license_ids).difference(before_license_ids))
        assert len(new_license_ids) == 1
        license = License.objects.get(id=new_license_ids[0])
        license.email_verified = True
        license.save()
        return license

    def test_query_workbooks(self) -> None:
        license = self._create_verified_license()
        self.assertEqual(
            graphql_query("query {workbooks{id}}", "example.com", license.key),
            {"data": {"workbooks": []}},
        )

        workbook = _create_workbook(license)
        self.assertEqual(
            graphql_query("query {workbooks{id}}", "example.com", license.key),
            {"data": {"workbooks": [{"id": str(workbook.id)}]}},
        )

    def test_query_workbook(self) -> None:
        license = self._create_verified_license()
        workbook = _create_workbook(license)
        self.assertEqual(
            graphql_query(
                """
                query {
                    workbook(workbookId:"%s") {
                      id
                    }
                }"""
                % workbook.id,
                "example.com",
                license.key,
            ),
            {"data": {"workbook": {"id": str(workbook.id)}}},
        )

        # bob can't access joe's workbook
        license2 = self._create_verified_license(email="bob@example.com")
        self.assertEqual(
            graphql_query(
                """
                query {
                    workbook(workbookId:"%s") {
                      id
                    }
                }"""
                % workbook.id,
                "example.com",
                license2.key,
                suppress_errors=True,
            ),
            {"data": {"workbook": None}},
        )

    def test_query_workbooks_multiple_users(self) -> None:
        license1 = self._create_verified_license(email="joe@example.com")
        license2 = self._create_verified_license(email="joe2@example.com")
        self.assertEqual(
            graphql_query("query {workbooks{id}}", "example.com", license1.key),
            {"data": {"workbooks": []}},
        )
        self.assertEqual(
            graphql_query("query {workbooks{id}}", "example.com", license2.key),
            {"data": {"workbooks": []}},
        )

        workbook = _create_workbook(license1)
        self.assertEqual(
            graphql_query("query {workbooks{id}}", "example.com", license1.key),
            {"data": {"workbooks": [{"id": str(workbook.id)}]}},
        )
        self.assertEqual(
            graphql_query("query {workbooks{id}}", "example.com", license2.key),
            {"data": {"workbooks": []}},
        )

    def test_create_workbook(self) -> None:
        self.assertEqual(Workbook.objects.count(), 0)
        license = self._create_verified_license()
        data = graphql_query(
            """
            mutation {
                create_workbook: createWorkbook {
                    workbook{revision, id, workbookJson}
                }
            }
            """,
            "example.com",
            license.key,
        )
        self.assertCountEqual(
            data["data"]["create_workbook"]["workbook"].keys(),
            ["revision", "id", "workbookJson"],
        )
        self.assertEqual(Workbook.objects.count(), 1)
        self.assertEqual(Workbook.objects.get().license, license)

    def test_create_workbook_unlicensed_domain(self) -> None:
        self.assertEqual(Workbook.objects.count(), 0)
        license = self._create_verified_license()
        data = graphql_query(
            """
            mutation {
                create_workbook: createWorkbook {
                    workbook{revision, id, workbookJson}
                }
            }
            """,
            "not-licensed.com",
            license.key,
            suppress_errors=True,
        )
        self.assertEqual(data["data"]["create_workbook"], None)
        self.assertEqual(Workbook.objects.count(), 0)

    def test_set_cell_input(self) -> None:
        license = self._create_verified_license(email="joe@example.com")
        workbook = _create_workbook(license)

        data = graphql_query(
            """
            mutation SetCellWorkbook($workbook_id: String!) {
                setCellInput(workbookId: $workbook_id, sheetId: 1, ref: "A1", input: "$2.50") { workbook { id } }
                output: setCellInput(workbookId: $workbook_id, sheetId: 1, row: 1, col: 2, input: "=A1*2") {
                    workbook {
                        id
                        sheet(sheetId: 1) {
                            id
                            B1: cell(ref: "B1") {
                                formattedValue
                                value {
                                    number
                                }
                                formula
                            }
                        }
                    }
                }
            }
            """,
            "example.com",
            license.key,
            {"workbook_id": str(workbook.id)},
        )
        self.assertEqual(
            data["data"]["output"],
            {
                "workbook": {
                    "id": str(workbook.id),
                    "sheet": {
                        "id": 1,
                        "B1": {"formattedValue": "$5.00", "value": {"number": 5}, "formula": "=A1*2"},
                    },
                },
            },
        )

        workbook.refresh_from_db()
        self.assertEqual(workbook.calc.sheets[0]["B1"].value, 5)

        # confirm that "other" license can't modify the workbook
        license_other = self._create_verified_license(email="other@example.com")
        data = graphql_query(
            """
            mutation {
                set_cell_input: setCellInput(workbookId:"%s", sheetId:1, ref: "A1", input: "100") {
                    workbook{ id }
                }
            }
            """
            % str(workbook.id),
            "example.com",
            license_other.key,
            suppress_errors=True,
        )
        self.assertIsNone(data["data"]["set_cell_input"])

    def test_save_workbook(self) -> None:
        license = self._create_verified_license()
        workbook = _create_workbook(license)
        self.assertEqual(workbook.revision, 1)

        new_json = equalto.new().json

        data = graphql_query(
            """
            mutation SaveWorkbook($workbook_json: String!) {
                save_workbook: saveWorkbook(workbookId: "%s", workbookJson: $workbook_json) {
                    revision
                }
            }
            """
            % str(workbook.id),
            "example.com",
            license.key,
            {"workbook_json": new_json},
        )
        self.assertEqual(data["data"]["save_workbook"], {"revision": 2})
        workbook.refresh_from_db()
        self.assertEqual(workbook.revision, 2)
        self.assertEqual(workbook.workbook_json, new_json)

    def test_save_workbook_invalid_json(self) -> None:
        license = self._create_verified_license()
        workbook = _create_workbook(license)
        old_workbook_json = workbook.workbook_json

        data = graphql_query(
            """
            mutation SaveWorkbook($workbook_json: String!) {
                save_workbook: saveWorkbook(workbookId: "%s", workbookJson: $workbook_json) {
                    revision
                }
            }
            """
            % str(workbook.id),
            "example.com",
            license.key,
            {"workbook_json": "not really a workbook JSON"},
            suppress_errors=True,
        )
        self.assertEqual(data["data"], {"save_workbook": None})
        workbook.refresh_from_db()
        self.assertEqual(workbook.revision, 1)
        self.assertEqual(workbook.workbook_json, old_workbook_json)

    def test_save_workbook_invalid_license(self) -> None:
        license = self._create_verified_license()
        workbook = _create_workbook(license)
        self.assertEqual(workbook.revision, 1)

        license2 = self._create_verified_license(email="bob@example.com")
        data = graphql_query(
            """
            mutation {
                save_workbook: saveWorkbook(workbookId: "%s", workbookJson: "{ }") {
                    revision
                }
            }
            """
            % str(workbook.id),
            "example.com",
            license2.key,
            suppress_errors=True,
        )
        self.assertEqual(data["data"]["save_workbook"], None)
        workbook.refresh_from_db()
        self.assertEqual(workbook.revision, 1)

    def test_save_workbook_unlicensed_domain(self) -> None:
        license = self._create_verified_license()
        workbook = _create_workbook(license)
        self.assertEqual(workbook.revision, 1)

        data = graphql_query(
            """
            mutation {
                save_workbook: saveWorkbook(workbookId: "%s", workbookJson: "{ }") {
                    revision
                }
            }
            """
            % str(workbook.id),
            "not-licensed.com",
            license.key,
            suppress_errors=True,
        )
        self.assertEqual(data["data"]["save_workbook"], None)
        workbook.refresh_from_db()
        self.assertEqual(workbook.revision, 1)

    def test_query_workbook_sheets(self) -> None:
        license = self._create_verified_license()
        workbook = _create_workbook(license, {"Calculation": {}, "Data": {}})

        self.assertEqual(
            graphql_query(
                """
                query {
                    workbook(workbookId: "%s") {
                        sheets {
                            id
                            name
                        }
                    }
                }"""
                % workbook.id,
                "example.com",
                license.key,
            ),
            {
                "data": {
                    "workbook": {
                        "sheets": [
                            {"id": 1, "name": "Calculation"},
                            {"id": 2, "name": "Data"},
                        ],
                    },
                },
            },
        )

    def test_query_workbook_sheet(self) -> None:
        license = self._create_verified_license()
        workbook = _create_workbook(license, {"FirstSheet": {}, "Calculation": {}, "Data": {}, "LastSheet": {}})

        self.assertEqual(
            graphql_query(
                """
                query {
                    workbook(workbookId: "%s") {
                        sheet_2: sheet(sheetId: 2) {
                            id
                            name
                        }
                        sheet_data: sheet(name: "Data") {
                            id
                            name
                        }
                    }
                }"""
                % workbook.id,
                "example.com",
                license.key,
            ),
            {
                "data": {
                    "workbook": {
                        "sheet_2": {"id": 2, "name": "Calculation"},
                        "sheet_data": {"id": 3, "name": "Data"},
                    },
                },
            },
        )

    def test_query_cell(self) -> None:
        license = self._create_verified_license()
        workbook = _create_workbook(license, {"Sheet": {"A1": "$2.50", "A2": "foobar", "A3": "true", "A4": "=2+2*2"}})

        self.assertEqual(
            graphql_query(
                """
                query {
                    workbook(workbookId: "%s") {
                        sheet(name: "Sheet") {
                            id
                            A1: cell(ref: "A1") {
                                formattedValue
                                value {
                                    text
                                    number
                                    boolean
                                }
                                type
                                format
                                formula
                            }
                            A2: cell(row: 2, col: 1) {
                                formattedValue
                                value {
                                    text
                                    number
                                    boolean
                                }
                                type
                                format
                                formula
                            }
                            A3: cell(col: 1, row: 3) {
                                formattedValue
                                value {
                                    text
                                    number
                                    boolean
                                }
                                type
                                format
                                formula
                            }
                            A4: cell(ref: "A4") {
                                formattedValue
                                value {
                                    text
                                    number
                                    boolean
                                }
                                type
                                format
                                formula
                            }
                            empty_cell: cell(ref: "A42") {
                                formattedValue
                            }
                        }
                    }
                }"""
                % workbook.id,
                "example.com",
                license.key,
            ),
            {
                "data": {
                    "workbook": {
                        "sheet": {
                            "id": 1,
                            "A1": {
                                "formattedValue": "$2.50",
                                "value": {"text": None, "number": 2.5, "boolean": None},
                                "type": "number",
                                "format": "$#,##0.00",
                                "formula": None,
                            },
                            "A2": {
                                "formattedValue": "foobar",
                                "value": {"text": "foobar", "number": None, "boolean": None},
                                "type": "text",
                                "format": "general",
                                "formula": None,
                            },
                            "A3": {
                                "formattedValue": "TRUE",
                                "value": {"text": None, "number": None, "boolean": True},
                                "type": "logical_value",
                                "format": "general",
                                "formula": None,
                            },
                            "A4": {
                                "formattedValue": "6",
                                "value": {"text": None, "number": 6.0, "boolean": None},
                                "type": "number",
                                "format": "general",
                                "formula": "=2+2*2",
                            },
                            "empty_cell": {"formattedValue": ""},
                        },
                    },
                },
            },
        )

        # confirm that None is returned if args list is invalid
        calls = [
            'cell(ref: "A4", col: 1)',
            'cell(ref: "A5", row: 1)',
            "cell(col: 1)",
            "cell(row: 1)",
            'cell(ref: "Sheet!A1")',
            'cell(ref: "InvalidRef")',
            "cell(row: 1, col: 0)",
        ]
        for call in calls:
            self.assertEqual(
                graphql_query(
                    """
                    query {
                        workbook(workbookId: "%s") {
                            sheet(name: "Sheet") {
                                cell: %s { formattedValue }
                            }
                        }
                    }"""
                    % (workbook.id, call),
                    "example.com",
                    license.key,
                    suppress_errors=True,
                ),
                {"data": {"workbook": {"sheet": None}}},
            )