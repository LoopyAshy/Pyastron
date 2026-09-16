# PyAstron

PyAstron embeds the Astron networking server in a Python extension module. The
Windows wheels use Python's stable ABI with a Python 3.7 minimum, so the same
wheel can be imported by CPython 3.7 and newer CPython 3.x releases, including
the Python 3.10 runtime used by Anachronome.

## Install from GitHub Releases

PyAstron does not need to be published to PyPI. Pin the exact release asset in
your application's `requirements.txt`:

```text
pyastron @ https://github.com/LoopyAshy/Pyastron/releases/download/v0.1.1/pyastron-0.1.1-cp37-abi3-win_amd64.whl ; sys_platform == "win32" and platform_machine == "AMD64"
```

Then install it normally:

```powershell
python -m pip install -r requirements.txt
```

`twine` is not involved in this installation flow. It is a publishing client
for package indexes; `pip` can install a wheel directly from a GitHub Release.

## Use

```python
import pyastron

astron = pyastron.create("config/astrond.yml")
astron.start()

# Later:
astron.shutdown()
```

The module also exposes `start_astron_direct` and `close_astron_direct` for
callers that manage the worker process themselves.

## Build a Windows wheel locally

The supported release target is 64-bit Windows. Install Rust, Visual Studio's
C++ build tools, Python, and vcpkg, then run:

```powershell
$env:VCPKG_ROOT = "C:\vcpkg"
$env:VCPKGRS_TRIPLET = "x64-windows-static-md"
& "$env:VCPKG_ROOT\vcpkg.exe" install libuv:x64-windows-static-md yaml-cpp:x64-windows-static-md boost-icl:x64-windows-static-md
python -m pip install "maturin>=1.7,<2"
python -m maturin build --release --locked --out dist
```

Clone with submodules (or initialize them before building):

```powershell
git clone --recurse-submodules https://github.com/LoopyAshy/Pyastron.git
```

## Release

Push a tag that exactly matches the Cargo package version, for example
`v0.1.1`. GitHub Actions builds the wheel, verifies imports on Python 3.7 and
3.10, and attaches it to a GitHub Release. Pull requests and ordinary pushes
build and test the wheel without publishing a release.
