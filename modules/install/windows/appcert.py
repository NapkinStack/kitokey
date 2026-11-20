"""The certification kit's report, said in a few lines (D77).

appcert.exe writes an XML report: REPORT, its OVERALL_RESULT and VERSION, then REQUIREMENT > TEST,
each with NAME, INDEX, OPTIONAL, its MESSAGE TEXTs and its RESULT — as written on GitHub's Windows
runner by the kit 10.0.26100.9457 (tests/appcert-report.xml, cut from it). Run as
`python appcert.py <report.xml>`, it prints them; a report it cannot read is said in one line.

Its exit is the run's verdict (D102): 0 when no required test failed — an optional test failed is
named, never blocking —, 1 when a required test did not pass, 2 when no report was read or it holds
no required test: a kit that did not run, or tested nothing, is never a pass.
"""
import sys
from xml.etree import ElementTree


def said(path):
    try:
        report = ElementTree.parse(path).getroot()
    except (OSError, ElementTree.ParseError) as error:
        return [f"certification kit: no report read — {error}"]
    if report.tag != "REPORT":
        return [f"certification kit: no report read — {path} holds <{report.tag}>, not the kit's <REPORT>"]
    tests = list(report.iter("TEST"))
    failed = [test for test in tests if (test.findtext("RESULT") or "").strip() != "PASS"]
    lines = [f"certification kit {report.get('VERSION')}: {report.get('OVERALL_RESULT')} overall, "
             f"{len(tests)} tests, {len(failed)} failed"]
    for test in failed:
        result = (test.findtext("RESULT") or "no result").strip()
        required = "optional" if test.get("OPTIONAL") == "TRUE" else "required"
        messages = [message.get("TEXT", "") for message in test.iter("MESSAGE")]
        first = f": {len(messages)} messages, the first: {messages[0]}" if messages else ""
        lines.append(f"  {result}  {test.get('NAME')} (test {test.get('INDEX')}, {required}){first}")
    return lines


def verdict(path):
    """0: no required test failed; 1: a required test did not pass; 2: no report read, or none required tested."""
    try:
        report = ElementTree.parse(path).getroot()
    except (OSError, ElementTree.ParseError):
        return 2
    if report.tag != "REPORT":
        return 2
    required = [test for test in report.iter("TEST") if test.get("OPTIONAL") != "TRUE"]
    # A kit that tested nothing required is no pass.
    if not required:
        return 2
    return 1 if any((test.findtext("RESULT") or "").strip() != "PASS" for test in required) else 0


if __name__ == "__main__":
    print("\n".join(said(sys.argv[1])))
    sys.exit(verdict(sys.argv[1]))
