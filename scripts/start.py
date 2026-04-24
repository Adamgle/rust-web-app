# from urllib.parse import urlparse

import shutil
import subprocess
import sys
from typing import Dict, Optional
from dotenv import dotenv_values
import pathlib


# Relative to cwd.
# The env is created on the frontend, we want to copy those in the server, have them the same.
# Probably that is not the best way to do it, maybe we should not share everything between the 2, but for now it is ok.
DOTENV_PATH = pathlib.Path("./.env")
# Relative to the app root.
FRONTEND_ENV_PATH = pathlib.Path("./frontend/stocked/.env")
FRONTEND_PATH = pathlib.Path("./frontend/stocked")


def start():
    # Check if frontend path exists and .env
    if not FRONTEND_PATH.exists():
        raise Exception(f"Frontend code got possibly migrated: {FRONTEND_PATH}")

    envs = dotenv_values(DOTENV_PATH, verbose=True)
    frontend_envs = dotenv_values(FRONTEND_ENV_PATH, verbose=True)

    # !!! NOTE: That code can only be used in development, envs should not be shared from server to client.

    # Detect merge conflicts
    client_public_envs: Dict[str, str | None] = dict()

    try:
        # Order insensitive comparison, dicts are different, conflicts detected.

        all_keys = set(envs.keys()) | set(frontend_envs.keys())

        exception: Optional[Exception] = None
        for key in all_keys:
            if key.startswith("NEXT_PUBLIC_"):
                if (
                    key in envs
                    and key in frontend_envs
                    and envs[key] != frontend_envs[key]
                ):
                    exception = Exception(
                        f"Public env variables exists in server and client .env file, review the conflict."
                    )

                    # This case takes precedences of the .env from the server to the client, user can review the changed before merging
                    client_public_envs[key] = envs[key]
                elif key in envs:
                    client_public_envs[key] = envs[key]
                elif key in frontend_envs:
                    client_public_envs[key] = frontend_envs[key]
            else:
                if exception is not None:
                    raise exception

        if dict(envs) != dict(frontend_envs):
            # Check if the envs in the client are the same as the one in the server

            diffs = {}

            for key in all_keys:
                # Do not consider the NEXT_PUBLIC_ as they are already handled.
                if not key.startswith("NEXT_PUBLIC_") and envs.get(
                    key
                ) != frontend_envs.get(key):
                    diffs[key] = (envs.get(key), frontend_envs.get(key))

            if diffs:
                raise Exception(
                    f"Differences found between server: {DOTENV_PATH} and client: {FRONTEND_ENV_PATH} files, review the conflicts: "
                    + ",".join(f"{k}: {v[0]} != {v[1]}" for k, v in diffs.items())
                )
    except Exception as e:
        print(f"[WARN]: {e}")
        choice = (
            input("Proceed with merge from server to client? y/N: ").lower().strip()
        )
        if choice != "y":
            print("Merge aborted.")
            sys.exit(1)

    # NOTE: Since we are using bacon to run the server, the .env would not copy itself
    # on the recompilation, we would need to run the script again.

    # Try to remove client side variables from the server .env file.

    print(client_public_envs)

    if client_public_envs:
        with open(DOTENV_PATH, "r+", encoding="utf-8") as f:
            # Remove the public envs from the server .env, if any
            new_content = [
                (
                    ""
                    if any(
                        public_env
                        for public_env in client_public_envs
                        if line.startswith(public_env)
                    )
                    else line
                )
                for line in f.readlines()
            ]

            f.truncate(0)
            f.seek(0)
            new_content = "".join(new_content).strip()
            f.write(new_content)

    # The .env of the server is the source of truth, copy it to the client.
    shutil.copyfile(DOTENV_PATH, FRONTEND_ENV_PATH)

    with open(FRONTEND_ENV_PATH, "r+", encoding="utf-8") as f:
        original_content = f.read()
        f.seek(0)
        f.write(f"# [INFO]: Copied {DOTENV_PATH} to {FRONTEND_ENV_PATH}\r\n")

        # Write client side envs
        f.write("# [INFO]: Client side public variables\r\n")

        buffer = [
            f"{key}={value}"
            for key, value in client_public_envs.items()
            # if key not in frontend_envs
        ]
        buffer = "\n".join(buffer)

        if buffer and buffer.strip():
            f.write(buffer + "\r\n")

        f.write(original_content)

    # Run server
    # # bacon --job run
    # bacon --job run --watch rust-web-app

    # bacon --job run-long -- 5000
    # $frontendDir = "./frontend/stocked"

    # Start-Process powershell -WorkingDirectory $frontendDir -ArgumentList "-NoExit", "-Command", "npm run dev -- -p 3000"
    # Start-Process powershell -ArgumentList "-NoExit", "-Command", "npm run dev -- -p $env:CLIENT_PORT"

    # SERVER_URL, CLIENT_URL = envs.get("SERVER_URL"), envs.get("CLIENT_URL")
    SERVER_PORT, CLIENT_PORT = envs.get("SERVER_PORT"), envs.get("CLIENT_PORT")

    #  Start server in a new PowerShell window
    subprocess.Popen(
        [
            "powershell",
            "-NoExit",
            "-Command",
            f"Start-Process powershell -ArgumentList '-NoExit','-Command','bacon --job run-long -- {SERVER_PORT}'",
        ],
    )

    # Start client in a new PowerShell window
    subprocess.Popen(
        [
            "powershell",
            "-NoExit",
            "-Command",
            f"Start-Process powershell -WorkingDirectory {FRONTEND_PATH} -ArgumentList '-NoExit','-Command','npm run dev'",
        ]
    )


if __name__ == "__main__":
    start()
