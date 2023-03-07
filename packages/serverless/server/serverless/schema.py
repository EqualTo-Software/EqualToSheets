from typing import Any, Callable, Iterable, Self

import equalto.cell
import equalto.exceptions
import equalto.sheet
import equalto.workbook
import graphene
from graphene_django import DjangoObjectType
from graphql import GraphQLError

from serverless import log, models
from serverless.util import is_license_key_valid_for_host


def get_license(info: graphene.ResolveInfo) -> models.License:
    auth = info.context.META.get("HTTP_AUTHORIZATION", None)
    if auth is None or auth[:7] != "Bearer ":
        raise GraphQLError("Invalid license key")
    license_key = auth[7:]
    qs_license = models.License.objects.filter(key=license_key)
    if qs_license.count() == 1:
        return qs_license.get()
    log.error("Could not find license for license key %s, qs_license.count()=%s" % (license_key, qs_license.count()))
    raise GraphQLError("Invalid license key")


def _validate_license_for_workbook(info: graphene.ResolveInfo, workbook_id: str) -> None:
    license = get_license(info)
    workbook = models.Workbook.objects.get(id=workbook_id)
    if workbook.license != license:
        log.error("User %s cannot access workbook %s" % (license.email, workbook))
        raise GraphQLError("ERROR - user can't access workbook")
    origin = info.context.META.get("HTTP_ORIGIN")
    if not is_license_key_valid_for_host(license.key, origin):
        log.error("User %s cannot access workbook %s from origin %s" % (license.email, workbook, origin))
        raise GraphQLError("ERROR - user can't access workbook")


def validate_license_for_workbook_mutation(graphql_fn: Callable[..., Any]) -> Callable[..., Any]:
    def fn(
        cls: type[graphene.Mutation],
        root: Any,
        info: graphene.ResolveInfo,
        workbook_id: str,
        *args: Any,
        **kwargs: Any,
    ) -> Callable[..., Any]:
        _validate_license_for_workbook(info, workbook_id)
        return graphql_fn(cls, root, info, workbook_id, *args, **kwargs)

    return fn


def validate_license_for_workbook_query(graphql_fn: Callable[..., Any]) -> Callable[..., Any]:
    def fn(root: Any, info: graphene.ResolveInfo, workbook_id: str, *args: Any, **kwargs: Any) -> Callable[..., Any]:
        _validate_license_for_workbook(info, workbook_id)
        return graphql_fn(root, info, workbook_id, *args, **kwargs)

    return fn


CellType = graphene.Enum.from_enum(equalto.cell.CellType)


class CellValue(graphene.ObjectType):
    text = graphene.String()
    number = graphene.Float()
    boolean = graphene.Boolean()


class Cell(graphene.ObjectType):
    def __init__(self, calc_cell: equalto.cell.Cell) -> None:
        self._calc_cell = calc_cell

    formatted_value = graphene.String(required=True)
    value = graphene.Field(CellValue, required=True)
    format = graphene.String(required=True)
    type = graphene.Field(CellType, required=True)
    formula = graphene.String()

    def resolve_formatted_value(self, info: graphene.ResolveInfo) -> str:
        return str(self._calc_cell)

    def resolve_value(self, info: graphene.ResolveInfo) -> CellValue:
        value = self._calc_cell.value
        if isinstance(value, bool):
            return CellValue(boolean=value)
        elif isinstance(value, (int, float)):
            return CellValue(number=float(value))
        elif isinstance(value, str):
            return CellValue(text=value)
        else:
            raise Exception(f"Unrecognized {type(value)=}")

    def resolve_format(self, info: graphene.ResolveInfo) -> str:
        return self._calc_cell.style.format

    def resolve_type(self, info: graphene.ResolveInfo) -> equalto.cell.CellType:
        return self._calc_cell.type

    def resolve_formula(self, info: graphene.ResolveInfo) -> str | None:
        return self._calc_cell.formula


class Sheet(graphene.ObjectType):
    def __init__(self, calc_sheet: equalto.sheet.Sheet) -> None:
        self._calc_sheet = calc_sheet

    name = graphene.String()
    id = graphene.Int()

    def resolve_name(self, info: graphene.ResolveInfo) -> str:
        return self._calc_sheet.name

    def resolve_id(self, info: graphene.ResolveInfo) -> int:
        return self._calc_sheet.sheet_id

    cell = graphene.Field(Cell, required=True, ref=graphene.String(), row=graphene.Int(), col=graphene.Int())

    def resolve_cell(
        self,
        info: graphene.ResolveInfo,
        ref: str | None = None,
        row: int | None = None,
        col: int | None = None,
    ) -> Cell:
        if ref is not None and row is None and col is None:
            return Cell(calc_cell=self._calc_sheet[ref])
        elif ref is None and row is not None and col is not None:
            return Cell(calc_cell=self._calc_sheet.cell(row, col))
        else:
            log.error("ERROR - ref/row/col")
            raise GraphQLError("ERROR - ref/row/col")


class Workbook(DjangoObjectType):
    class Meta:
        model = models.Workbook
        fields = (
            "id",
            "workbook_json",
            "create_datetime",
            "modify_datetime",
            "revision",
        )

    sheet = graphene.Field(Sheet, sheet_id=graphene.Int(), name=graphene.String())
    sheets = graphene.List(Sheet)

    def resolve_sheet(self, info: graphene.ResolveInfo, sheet_id: int | None = None, name: str | None = None) -> Sheet:
        if sheet_id is not None and name is None:
            return Sheet(calc_sheet=self.calc.sheets._get_sheet(sheet_id))  # noqa: WPS437
        elif sheet_id is None and name is not None:
            return Sheet(calc_sheet=self.calc.sheets[name])
        else:
            log.error("ERROR - name/id")
            raise Exception("ERROR - name/id")

    def resolve_sheets(self, info: graphene.ResolveInfo) -> Iterable[Sheet]:
        log.info("Origin: %s" % info.context.META.get("Origin"))
        yield from (Sheet(calc_sheet=sheet) for sheet in self.calc.sheets)


class Query(graphene.ObjectType):
    workbook = graphene.Field(Workbook, workbook_id=graphene.String(required=True))
    workbooks = graphene.List(Workbook)

    @validate_license_for_workbook_query
    def resolve_workbook(self, info: graphene.ResolveInfo, workbook_id: str) -> models.Workbook:
        return models.Workbook.objects.get(id=workbook_id)

    def resolve_workbooks(self, info: graphene.ResolveInfo) -> Iterable[models.Workbook]:
        license = get_license(info)
        return models.Workbook.objects.filter(license=license)


class SaveWorkbook(graphene.Mutation):
    class Arguments:
        workbook_id = graphene.String()
        workbook_json = graphene.String()

    revision = graphene.Int()

    @classmethod
    @validate_license_for_workbook_mutation
    def mutate(
        cls,
        root: Any,
        info: graphene.ResolveInfo,
        workbook_id: str,
        workbook_json: str,
    ) -> Self:
        workbook = models.Workbook.objects.select_for_update().get(id=workbook_id)

        try:
            workbook.workbook_json = equalto.loads(workbook_json).json
        except equalto.exceptions.WorkbookError:
            raise GraphQLError("Could not parse workbook JSON")

        workbook.revision += 1
        workbook.save(update_fields=["workbook_json", "revision"])

        return SaveWorkbook(revision=workbook.revision)


class SetCellInput(graphene.Mutation):
    # simulates entering text in the spreadsheet widget
    class Arguments:
        workbook_id = graphene.String(required=True)
        sheet_id = graphene.Int()
        sheet_name = graphene.String()
        ref = graphene.String()
        row = graphene.Int()
        col = graphene.Int()

        input = graphene.String(required=True)

    workbook = graphene.Field(Workbook, required=True)

    @classmethod
    @validate_license_for_workbook_mutation
    def mutate(
        cls,
        root: Any,
        info: graphene.ResolveInfo,
        workbook_id: str,
        input: str,
        sheet_id: int | None = None,
        sheet_name: str | None = None,
        ref: str | None = None,
        row: int | None = None,
        col: int | None = None,
    ) -> Self:
        workbook = models.Workbook.objects.select_for_update().get(id=workbook_id)

        if sheet_name is not None:
            assert sheet_id is None
            sheet = workbook.calc.sheets[sheet_name]
        else:
            assert sheet_id is not None
            sheet = workbook.calc.sheets._get_sheet(sheet_id)  # noqa: WPS437

        if ref is not None:
            assert row is None and col is None
            cell = sheet[ref]
        else:
            assert row is not None and col is not None
            cell = sheet.cell(row, col)

        cell.set_user_input(input)

        workbook.workbook_json = workbook.calc.json
        workbook.revision += 1
        workbook.save(update_fields=["workbook_json", "revision"])
        return SetCellInput(workbook=workbook)


class CreateWorkbook(graphene.Mutation):
    # The class attributes define the response of the mutation
    workbook = graphene.Field(Workbook, required=True)

    @classmethod
    def mutate(cls, root: Any, info: graphene.ResolveInfo) -> Self:
        log.info("MUTATE CREATE WORKBOOK")
        # The /graphiql client doesn't seem to be inserting the Authorization header
        license = get_license(info)
        origin = info.context.META.get("HTTP_ORIGIN")
        log.info(
            "CreateWorkbook: auth=%s, license=%s, origin=%s"
            % (info.context.META.get("HTTP_AUTHORIZATION"), license, origin),
        )
        if not is_license_key_valid_for_host(license.key, origin):
            log.error("License key %s is not valid for %s." % (license.key, origin))
            raise GraphQLError("Invalid license key")
        workbook = models.Workbook(license=license)
        workbook.save()
        # Notice we return an instance of this mutation
        return CreateWorkbook(workbook=workbook)


class Mutation(graphene.ObjectType):
    save_workbook = SaveWorkbook.Field()
    create_workbook = CreateWorkbook.Field()
    set_cell_input = SetCellInput.Field()


schema = graphene.Schema(query=Query, mutation=Mutation)