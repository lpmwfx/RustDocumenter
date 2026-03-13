@echo off
:: slintman.bat — open rustdoc-viewer for the nearest man/ directory (Slint projects)
::
:: Usage:
::   slintman              walk up from CWD to find man/MANIFEST.json
::   slintman <PATH>       open viewer for given project/man dir
::   slintman gen          generate man/ for CWD first, then open
::   slintman check        check doc coverage for CWD

call rustman.bat %*
