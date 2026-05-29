#!/usr/bin/env python3
"""
Script de verificacao: garante que todos os 290 TCs do spec.md
foram gerados nos arquivos atomicos em test_cases/atomic/.
"""

import re
import sys
from pathlib import Path

def extract_tcs_from_spec(spec_path: Path) -> set:
    """Extrai todos os TC-XXX mencionados no spec.md."""
    text = spec_path.read_text(encoding="utf-8")
    matches = re.findall(r'TC-(\d{3})', text)
    return set(int(m) for m in matches)

def extract_tcs_from_atomic_files(atomic_dir: Path) -> set:
    """Extrai todos os TC-XXX dos arquivos .md em test_cases/atomic/."""
    tcs = set()
    for md_file in atomic_dir.glob("*.md"):
        text = md_file.read_text(encoding="utf-8")
        matches = re.findall(r'TC-(\d{3})', text)
        tcs.update(int(m) for m in matches)
    return tcs

def extract_tcs_from_test_file(test_path: Path) -> set:
    """Extrai TCs do arquivo de teste Rust."""
    text = test_path.read_text(encoding="utf-8")
    # Padrao TC-XXX em comentarios/strings
    matches = re.findall(r'TC-(\d{3})', text)
    # Padrao fn tc_XXX_ em nomes de funcoes
    fn_matches = re.findall(r'fn tc_(\d{3})_', text)
    return set(int(m) for m in matches) | set(int(m) for m in fn_matches)

def main():
    spec_path = Path("specs/006-big-test-plan/spec.md")
    atomic_dir = Path("test_cases/atomic")
    test_path = Path("tests/big_test_plan.rs")

    if not spec_path.exists():
        print("ERRO: {} nao encontrado!".format(spec_path))
        sys.exit(1)

    spec_tcs = extract_tcs_from_spec(spec_path)
    expected_range = set(range(1, 291))

    print("=" * 60)
    print("VERIFICACAO DE TEST CASES - Feature 006-big-test-plan")
    print("=" * 60)
    print("\nspec.md: {} TCs unicos encontrados".format(len(spec_tcs)))

    missing_in_spec = expected_range - spec_tcs
    if missing_in_spec:
        print("ATENCAO: TCs faltando no spec.md: {}".format(sorted(missing_in_spec)))
    else:
        print("OK: spec.md cobre todos os TCs de 001 a 290")

    atomic_tcs = extract_tcs_from_atomic_files(atomic_dir)
    print("\ntest_cases/atomic/: {} TCs unicos encontrados".format(len(atomic_tcs)))

    test_tcs = set()
    if test_path.exists():
        test_tcs = extract_tcs_from_test_file(test_path)
        print("tests/big_test_plan.rs: {} TCs unicos encontrados".format(len(test_tcs)))
    else:
        print("ATENCAO: tests/big_test_plan.rs nao encontrado")

    missing_atomic = expected_range - atomic_tcs
    missing_test = expected_range - test_tcs

    print("\n" + "=" * 60)
    print("RESULTADO")
    print("=" * 60)

    if not missing_atomic:
        print("OK: TODOS os 290 TCs estao presentes nos arquivos atomicos!")
    else:
        print("FALTAM {} TCs nos arquivos atomicos:".format(len(missing_atomic)))
        for tc in sorted(missing_atomic):
            print("   - TC-{:03d}".format(tc))

    if test_path.exists():
        if not missing_test:
            print("OK: TODOS os 290 TCs estao presentes no arquivo Rust!")
        else:
            print("FALTAM {} TCs no arquivo Rust:".format(len(missing_test)))
            for tc in sorted(missing_test):
                print("   - TC-{:03d}".format(tc))

    print("\n" + "=" * 60)
    print("DETALHAMENTO POR ARQUIVO ATOMICO")
    print("=" * 60)
    for md_file in sorted(atomic_dir.glob("*.md")):
        text = md_file.read_text(encoding="utf-8")
        matches = re.findall(r'TC-(\d{3})', text)
        unique = set(int(m) for m in matches)
        print("  {:40s} -> {:3d} TCs".format(md_file.name, len(unique)))

    all_matches = []
    for md_file in atomic_dir.glob("*.md"):
        text = md_file.read_text(encoding="utf-8")
        # Ignora ranges como "TC-001 a TC-015" para evitar falsos positivos
        text_no_ranges = re.sub(r'TC-\d{3}\s+a\s+TC-\d{3}', '', text)
        matches = re.findall(r'TC-(\d{3})', text_no_ranges)
        for m in matches:
            all_matches.append((int(m), md_file.name))

    tc_counts = {}
    for tc, fname in all_matches:
        tc_counts.setdefault(tc, []).append(fname)

    duplicates = {tc: files for tc, files in tc_counts.items() if len(files) > 1}
    if duplicates:
        print("\nATENCAO: TCs duplicados entre arquivos:")
        for tc, files in sorted(duplicates.items()):
            print("   TC-{:03d}: {}".format(tc, ', '.join(files)))
    else:
        print("\nOK: Nenhum TC duplicado entre arquivos atomicos")

    if missing_atomic or missing_test:
        sys.exit(1)
    sys.exit(0)

if __name__ == "__main__":
    main()
