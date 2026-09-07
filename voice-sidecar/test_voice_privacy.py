import ast
from pathlib import Path

SOURCE = Path(__file__).with_name("jarvis_wakeword.py").read_text()
TREE = ast.parse(SOURCE)

def test_sidecar_contains_no_network_imports():
    forbidden = {"requests", "urllib", "http", "aiohttp", "websockets"}
    imported = set()
    for node in ast.walk(TREE):
        if isinstance(node, ast.Import): imported.update(alias.name.split(".")[0] for alias in node.names)
        if isinstance(node, ast.ImportFrom) and node.module: imported.add(node.module.split(".")[0])
    assert not imported.intersection(forbidden)

def test_runtime_network_is_deterministically_denied():
    assert 'event.startswith("socket.")' in SOURCE
    assert "sys.addaudithook(_deny_network)" in SOURCE

def test_ambient_audio_is_not_written_in_callback():
    callback = next(n for n in ast.walk(TREE) if isinstance(n, ast.FunctionDef) and n.name == "callback")
    calls = {getattr(n.func, "attr", "") for n in ast.walk(callback) if isinstance(n, ast.Call)}
    assert "write" not in calls and "writeframes" not in calls

def test_temp_audio_is_only_created_after_capture_activation():
    functions = {n.name: ast.unparse(n) for n in TREE.body if isinstance(n, (ast.FunctionDef, ast.ClassDef))}
    assert "NamedTemporaryFile" not in functions.get("emit", "")
    assert "capture_command" in SOURCE and "NamedTemporaryFile" in SOURCE
