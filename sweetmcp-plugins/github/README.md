# github

[src](https://github.com/dylibso/mcp.run-servlets/tree/main/servlets/github)

You can interact with GitHub via various tools available in this plugin: branches, repo, gist, issues, files, etc...

## Usage

```json
{
    "plugins": [
        {
            "name": "github",
            "path": "oci://ghcr.io/tuananh/github-plugin:latest",
            "env": {
                "allowed_hosts": [
                    "api.github.com"
                ],
                "env_vars": {
                    "api-key": "ghp_xxxx"
                }
            }
        }
    ]
}
```