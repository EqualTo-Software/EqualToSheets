import re
from typing import Any

from serverless.log import error, info
from serverless.models import License, LicenseDomain


class LicenseKeyError(Exception):
    """A problem with the license key."""


def get_name_from_path(path: str) -> str:
    m = re.search(r"([^/\\]+$)", path)
    return m.group(0) if m else ""


def get_license(http_meta: dict[str, Any]) -> License:
    auth = http_meta.get("HTTP_AUTHORIZATION", None)
    if auth is None or auth[:7] != "Bearer ":
        raise LicenseKeyError("Invalid license key")
    license_key = auth[7:]
    qs_license = License.objects.filter(key=license_key)
    try:
        return qs_license.get()
    except (License.DoesNotExist, License.MultipleObjectsReturned):
        error(f"Could not find license for license key {license_key}, {qs_license.count()=}")
        raise LicenseKeyError("Invalid license key")


def is_license_key_valid_for_host(license_key: str, host: str | None) -> bool:
    # the license is valid if:
    #   1. There are no LicenseDomain records (ie it's a beta license key, which does not
    #      require a list of licensed domains), OR
    #   2. LicenseDomain contains a record for domain, OR
    #   3. LicenseDomain contains a record for the parent of domain, with *. at the start, OR

    # This will be removed post-beta
    host = "" if host is None else host
    assert isinstance(host, str)
    if host.startswith("http://"):
        host = host[len("http://") :]
    elif host.startswith("https://"):
        host = host[len("https://") :]
    domain = host.split(":")[0]
    parent = ".".join(domain.split(".")[1:])
    parent_wild_card = "*.%s" % parent
    info("license_key=%s, host=%s" % (license_key, host))
    license = License.objects.get(key=license_key)
    info("got license for %s, host=%s, email_verified=%s" % (license.email, host, license.email_verified))
    return license.email_verified and (
        # WARNING: during the beta, if a license has 0 domains then it works on all domains
        # TODO: remove this after the beta
        LicenseDomain.objects.filter(license=license).count() == 0
        or (
            # exact mach
            LicenseDomain.objects.filter(license=license, domain=domain).exists()
            or
            # parent with wild card
            LicenseDomain.objects.filter(license=license, domain=parent_wild_card).exists()
        )
    )
