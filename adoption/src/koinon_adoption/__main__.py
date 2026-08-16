"""`python -m koinon_adoption` — delegates to `cli.main`."""

from __future__ import annotations

import sys

from koinon_adoption.cli import main

if __name__ == "__main__":
    sys.exit(main())
