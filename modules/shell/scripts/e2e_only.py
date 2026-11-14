"""The scenarios a run of e2e_window.py plays, as pytest's `-k` chooses tests: those
KITOKEY_E2E_ONLY names (`name[,name…]`, a scenario's function), in the order main() calls them —
or, with none named, every one main() calls. A name main() does not call refuses the run.
Read without Selenium: run alone (`python3 scripts/e2e_only.py scripts/e2e_window.py`), it prints them."""
import ast
import os
import pathlib
import sys


def scenarios(source: str) -> list[str]:
    """The scenarios main() calls, in its order: each a statement `name(now)` of its own."""
    main = next(node for node in ast.parse(source).body if isinstance(node, ast.FunctionDef) and node.name == "main")
    return [
        statement.value.func.id
        for statement in main.body
        if isinstance(statement, ast.Expr) and isinstance(statement.value, ast.Call) and isinstance(statement.value.func, ast.Name)
        and [getattr(arg, "id", None) for arg in statement.value.args] == ["now"] and not statement.value.keywords
    ]


def chosen(source: str, only: str) -> list[str]:
    """Those `only` names, in main()'s order; all of them when it names none."""
    called = scenarios(source)
    asked = [name.strip() for name in only.split(",") if name.strip()]
    unknown = [name for name in asked if name not in called]
    if unknown:
        raise SystemExit(f"e2e: partial run refused — no scenario named {', '.join(unknown)}")
    return [name for name in called if name in asked] if asked else called


if __name__ == "__main__":
    print("\n".join(chosen(pathlib.Path(sys.argv[1]).read_text(), os.environ.get("KITOKEY_E2E_ONLY", ""))))
