# Authenticated user

Retrieve the account represented by the configured credentials:

```sh
hevy-rs --format json user get
```

The successful JSON object is the authenticated user's information. Use [common CLI guidance](common.md) for credential resolution and JSON error handling.

For current operation semantics and the API response shape, see the [official Hevy API user-information operation](https://api.hevyapp.com/docs/#/Users/get_v1_user_info). Do not rely on a copied payload or response schema.
