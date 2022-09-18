# openldbsvws-rs

Rust library for National Rail Enquiries' OpenLDBSVWS API.

This code is licensed under the WTFPL, but its output is not.
Any NRE data included in the output of this program is subject to these [terms and conditions](https://opendata.nationalrail.co.uk/terms).

## Getting started

You'll need a token for the OpenLDBSVWS API.

> ### Caution
> The OpenLDBSVWS API (also called OpenLDBWSSV) is not the same as the OpenLDBWS API.
> You need a token for the LDB Webservice (Staff Version), not the LDB Webservice (PV).
> Despite the name, you do not need to work for National Rail to use it.

Then, use the token in the CLI:

```bash
openldbsvws service -t <token> <rid>
```

Right now, you can only fetch service details.
More features are being implemented soon.
