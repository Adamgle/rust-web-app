### python ./scripts/start.py 

# # # bacon --job run
# # bacon --job run --watch rust-web-app

# bacon --job run-long -- 5000
# $frontendDir = "./frontend/stocked"

# # Flush the .env of the server inside the .env of the next.js .env.
# # We want to have one file of .env and keep them synchronized.
# # We need to detect conflicts between the .env file of the server and the client.
# # There cannot be a case where env is defined in the client, but not on the server, the reverse is permitted.
# Copy-Item -Path ".env" -Destination "$frontendDir/.env" -Force

# Start-Process powershell -WorkingDirectory $frontendDir -ArgumentList "-NoExit", "-Command", "npm run dev -- -p 3000"
# # Start-Process powershell -ArgumentList "-NoExit", "-Command", "npm run dev -- -p $env:CLIENT_PORT"