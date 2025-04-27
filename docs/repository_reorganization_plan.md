# Repository Reorganization Plan

## Current Issues
- Too many files at the root level
- Mixed file types (Python, Markdown, configuration, etc.)
- Lack of clear organization by purpose or functionality
- Difficult to navigate and find specific files

## Proposed Folder Structure

### Root Level (Keep Essential Files Only)
- README.md
- Cargo.toml
- Cargo.lock
- .gitignore
- TODO.md
- TESTS.md
- Trunk.toml
- index.html
- styles.css

### New Folder Structure

1. `/interpreters/`
   - anarchy.py
   - anarchy_simple.py
   - anarchy_simple_fixed.py
   - interpreter_guide.md

2. `/docs/` (Expand existing)
   - language_reference.md
   - All existing documentation
   - `/docs/project/`
     - competitive_analysis.md
     - comparison_matrix.md
     - anarchy_advantages_analysis.md
   - `/docs/budget/`
     - budget_justification.md
     - budget_categories_allocations.md
     - final_budget_justification.md
   - `/docs/strategy/`
     - community_building_strategy.md
     - multi_grant_funding_strategy.md
     - final_report.md

3. `/tests/` (Consolidate testing)
   - test_language.py
   - test.a.i
   - test_core_features.a.i
   - test_simple.a.i
   - test_recent_additions.a.i
   - test_input.txt
   - minimal_test.a.i
   - Move existing test_reports/ and test_output/ here
   - Move anarchy_test_suite/ here

4. `/benchmarks/`
   - benchmark_framework.py
   - run_benchmarks.py
   - run_benchmarks_optimized.py
   - Move benchmark_results/ here

5. `/tools/`
   - generate_sample_data.py
   - deploy_github_pages.sh

6. `/website/` (Expand existing)
   - website_hosting_options.md
   - All existing website files

7. `/community/` (Consolidate community resources)
   - Move existing community/ here
   - Move community_forum/ here

## Migration Strategy
1. Create all new directories
2. Move files to appropriate locations
3. Update any import statements or references
4. Test functionality after migration
5. Commit and push changes
