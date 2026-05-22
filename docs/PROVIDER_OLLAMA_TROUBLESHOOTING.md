# Provider Ollama Troubleshooting

This guide explains what to do when Smainer Desktop accepts an AI task but the bot reports an Ollama error such as:

```text
Compute failed: AI inference error: Client error '404 Not Found' for url 'http://localhost:11434/api/generate'
```

## What It Means

Smainer Desktop runs AI tasks through a local Ollama server on the same computer as the provider. The provider calls:

```text
http://localhost:11434/api/generate
```

If that endpoint returns `404 Not Found`, the desktop node cannot generate with the configured model. Common causes are:

- Ollama is installed but not fully running.
- The selected model is not installed in Ollama.
- Smainer Desktop is connected to a different Ollama runtime than the one you tested.
- Ollama was restarted or updated after Smainer Desktop already advertised AI capability.

Recent Smainer Desktop builds verify that configured models exist before advertising AI capability. If you are using an older build, update Smainer Desktop and restart the provider.

## Fix On Windows

Run these commands in PowerShell on the same computer that is running Smainer Desktop.

Check that Ollama is installed:

```powershell
ollama --version
```

Check that the model is installed:

```powershell
ollama list
```

If `llama3.1:8b` is missing, install it:

```powershell
ollama pull llama3.1:8b
```

Test generation through the Ollama CLI:

```powershell
ollama run llama3.1:8b "Reply with exactly: OK"
```

Expected output:

```text
OK
```

Test the exact HTTP API used by the Smainer provider:

```powershell
$body = @{ model="llama3.1:8b"; prompt="Reply with exactly: OK"; stream=$false } | ConvertTo-Json -Compress
Invoke-WebRequest -UseBasicParsing -Method Post http://localhost:11434/api/generate -ContentType "application/json" -Body $body -TimeoutSec 180
```

The response should contain:

```json
"response":"OK"
```

After this works, close and reopen Smainer Desktop, then start the provider again.

## Fix On Linux Or WSL

Run these commands on the same system where the provider process is running:

```bash
ollama --version
ollama list
ollama pull llama3.1:8b
ollama run llama3.1:8b "Reply with exactly: OK"
curl -sS http://localhost:11434/api/version
curl -sS http://localhost:11434/api/tags
curl -sS -X POST http://localhost:11434/api/generate \
  -H 'Content-Type: application/json' \
  -d '{"model":"llama3.1:8b","prompt":"Reply with exactly: OK","stream":false}'
```

After the API returns `OK`, restart Smainer Desktop or the provider daemon.

## Important: Same Machine, Same Runtime

Testing Ollama on another machine does not prove the desktop provider can use it. `localhost` always means "this computer" from the provider process point of view.

For example:

- Testing inside WSL may use a Linux Ollama service.
- Testing in Windows PowerShell may use a Windows Ollama service or a WSL relay.
- Testing on a remote server does not test the desktop provider machine.

Always test from the machine and environment where Smainer Desktop is running.

## If It Still Fails

Collect these details before opening an issue:

```powershell
ollama --version
ollama list
Invoke-WebRequest -UseBasicParsing http://localhost:11434/api/version
Invoke-WebRequest -UseBasicParsing http://localhost:11434/api/tags
```

Also include the Smainer Desktop version and the exact model selected in the AI setup screen.
