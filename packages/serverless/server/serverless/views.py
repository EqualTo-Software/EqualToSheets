from asyncio import sleep
from typing import Any

import equalto
from asgiref.sync import sync_to_async
from django.db import transaction
from django.http import (
    HttpRequest,
    HttpResponse,
    HttpResponseBadRequest,
    HttpResponseForbidden,
    HttpResponseNotAllowed,
    HttpResponseNotFound,
    JsonResponse,
)
from graphene_django.views import GraphQLView

from server import settings
from serverless.log import info
from serverless.models import License, LicenseDomain, Workbook
from serverless.schema import schema
from serverless.util import LicenseKeyError, get_license


# Create your views here.
def index(request: HttpRequest) -> HttpResponse:
    return HttpResponse("Hello from Python! %s, Workbooks.count: %s" % (equalto.__version__, Workbook.objects.count()))


def send_license_key(request: HttpRequest, _send_email: bool = True) -> HttpResponse:
    info("send_license_key(): headers=%s" % request.headers)
    if not settings.DEBUG and request.method != "POST":
        return HttpResponseNotAllowed("405: Method not allowed.")

    email = request.POST.get("email", None) or request.GET.get("email", None)
    if email is None:
        return HttpResponseBadRequest("You must specify the 'email' field.")
    if License.objects.filter(email=email).exists():
        return HttpResponseBadRequest("License key already created for '%s'." % email)
    domain_csv = request.POST.get("domains", "") or request.GET.get("domains", "")
    # WARNING: during the beta, if a license has 0 domains, then the license key will work on all domains
    # TODO: post-beta, we'll require that a license key requires one or more domains
    domains = list(filter(lambda s: s != "", map(lambda s: s.strip(), domain_csv.split(","))))

    # create license & license domains
    license = License(email=email)
    license.save()
    for domain in domains:
        license_domain = LicenseDomain(license=license, domain=domain)
        license_domain.save()

    info("license_id=%s, license key=%s" % (license.id, license.key))
    activation_url = "http://localhost:5000/activate-license-key/%s/" % license.id
    info("activation_url=%s" % activation_url)
    # TODO:
    #   0. Create License and LicenseDomains records
    #   1. send email with newly created license for activation (click on verification link)
    #   2. display instructions ("check email")

    return HttpResponse(
        'License key created: <a href="/activate-license-key/%s">activate %s</a>' % (license.id, license.key),
    )


def activate_license_key(request: HttpRequest, license_id: str) -> HttpResponse:
    license = License.objects.get(id=license_id)
    license.email_verified = True
    license.save()

    return HttpResponse("Your license key: %s. Code samples: ..." % license.key)


def graphql_view(request: HttpRequest, *args: Any, **kwargs: Any) -> HttpResponse:
    return GraphQLView.as_view(graphiql=True, schema=schema)(request, *args, **kwargs)


@transaction.non_atomic_requests
async def get_updated_workbook(request: HttpRequest, workbook_id: str, revision: int) -> HttpResponse:
    # TODO: Long poll design isn't the best but should be sufficient for the beta.
    try:
        license = await sync_to_async(lambda: get_license(request.META))()
    except LicenseKeyError:
        return HttpResponseForbidden("Invalid license")

    try:
        workbook = await Workbook.objects.aget(id=workbook_id, license=license)
    except Workbook.DoesNotExist:
        return HttpResponseNotFound("Requested workbook does not exist")

    if workbook.revision < revision:
        # the client's version is newer, that doesn't seem right
        return HttpResponseBadRequest()

    # TODO: We should check the license settings against the request HOST but we're skipping that in the beta.

    for _ in range(50):  # 50 attempts, one connection is open for up to ~10s
        workbook = await _get_updated_workbook(workbook_id, revision)
        if workbook is not None:
            return JsonResponse(workbook)
        await sleep(0.2)

    return HttpResponse(status=408)  # timeout


async def _get_updated_workbook(workbook_id: str, revision: int) -> dict[str, Any] | None:
    workbook = await Workbook.objects.filter(id=workbook_id, revision__gt=revision).order_by("revision").alast()
    if workbook is None:
        return None
    return {
        "revision": workbook.revision,
        "workbook_json": workbook.workbook_json,
    }
