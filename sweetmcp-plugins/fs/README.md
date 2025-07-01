# fs

An example plugin that implement filesystem operations.

## Usage

```json
{
  "plugins": [
    {
      "name": "fs",
      "path": "oci://ghcr.io/tuananh/fs-plugin:latest",
      "env": {
        "allowed_paths": ["/tmp"]
      }
    }
  ]
}

```
