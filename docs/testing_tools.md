# Testing Tools Documentation

This document provides comprehensive documentation for the Anarchy Inference Testing Tools, which include:

1. Record/Replay System
2. Automated Test Generation
3. Coverage Analysis
4. Performance Benchmarking

## Overview

The Anarchy Inference Testing Tools provide a complete testing infrastructure for developing, debugging, and optimizing Anarchy Inference code. These tools work together to enable a comprehensive testing workflow:

- **Record user interactions** with the Anarchy Inference interpreter
- **Generate tests** based on recorded sessions or templates
- **Analyze code coverage** to ensure thorough testing
- **Benchmark performance** to optimize token efficiency

## Record/Replay System

The Record/Replay System captures and reproduces interpreter sessions, enabling deterministic testing and debugging.

### Key Components

- **RecordingSession**: Captures interpreter events during code execution
- **ReplaySession**: Replays recorded events to reproduce execution
- **RecordingManager**: Manages multiple recordings and provides a unified interface

### Usage Examples

#### Recording a Session

```python
from record_replay.record_replay import RecordingSession

# Create a recording session
session = RecordingSession("my_session", interpreter)

# Start recording
session.start_recording()

# Execute code
interpreter.execute("""
λ⟨ test_function ⟩
    x ← 1
    y ← 2
    return x + y

result ← test_function()
""")

# Stop recording
session.stop_recording()

# Save the recording
session.save_recording("/path/to/recording.json")
```

#### Replaying a Session

```python
from record_replay.record_replay import ReplaySession

# Create a replay session
replay = ReplaySession("replay_session", interpreter)

# Load a recording
replay.load_recording("/path/to/recording.json")

# Replay the recording
replay.replay()

# Check if replay is complete
if replay.is_replay_complete():
    print("Replay completed successfully")
```

#### Using the Recording Manager

```python
from record_replay.record_replay import RecordingManager

# Create a recording manager
manager = RecordingManager("/path/to/recordings")

# Register recordings
manager.register_recording("session1", "/path/to/recording1.json")
manager.register_recording("session2", "/path/to/recording2.json")

# List available recordings
recordings = manager.list_recordings()
print(f"Available recordings: {recordings}")

# Replay a recording
manager.replay_recording("session1")
```

### Advanced Features

- **Conditional Breakpoints**: Pause replay when specific conditions are met
- **Event Filtering**: Focus on specific types of events during replay
- **Execution Comparison**: Compare original and replayed executions
- **Session Merging**: Combine multiple recordings into a single session

## Automated Test Generation

The Automated Test Generation system creates test cases automatically based on templates, fuzzing, or recorded sessions.

### Key Components

- **TestTemplate**: Defines parameterized test templates
- **TestGenerator**: Generates test cases from templates
- **Fuzzer**: Creates randomized inputs for test generation

### Usage Examples

#### Creating a Test Template

```python
from test_generation.test_generation import TestTemplate

# Create a test template
template = TestTemplate(
    name="arithmetic_test",
    template="""
    λ⟨ test_arithmetic ⟩
        x ← {{x_value}}
        y ← {{y_value}}
        expected ← {{expected}}
        result ← x + y
        assert(result == expected)
    
    test_arithmetic()
    """
)

# Render the template with values
values = {
    "x_value": 5,
    "y_value": 10,
    "expected": 15
}

rendered_test = template.render(values)
print(rendered_test)
```

#### Generating Tests

```python
from test_generation.test_generation import TestGenerator

# Create a test generator
generator = TestGenerator(interpreter)

# Add a template
generator.add_template(template)

# Generate tests
tests = generator.generate_tests(
    template_name="arithmetic_test",
    count=10,
    value_ranges={
        "x_value": (-100, 100),
        "y_value": (-100, 100)
    },
    derived_values={
        "expected": lambda values: values["x_value"] + values["y_value"]
    }
)

# Save the tests
generator.save_tests(tests, "/path/to/tests/")
```

#### Using the Fuzzer

```python
from test_generation.test_generation import Fuzzer

# Create a fuzzer
fuzzer = Fuzzer()

# Generate random values
int_value = fuzzer.generate_int(-100, 100)
float_value = fuzzer.generate_float(-1.0, 1.0)
string_value = fuzzer.generate_string(10)
bool_value = fuzzer.generate_bool()

# Generate structured data
dict_value = fuzzer.generate_dict({"key1": "int", "key2": "string"})
list_value = fuzzer.generate_list("int", 5)
```

### Advanced Features

- **Property-Based Testing**: Generate tests based on properties that should hold
- **Mutation Testing**: Create tests by mutating existing code
- **Symbolic Execution**: Generate tests that cover specific execution paths
- **Test Minimization**: Reduce test cases while maintaining coverage

## Coverage Analysis

The Coverage Analysis system measures how thoroughly tests exercise code, identifying untested areas.

### Key Components

- **CoverageAnalyzer**: Instruments code and collects coverage data
- **CoverageReporter**: Generates reports from coverage data
- **ExecutionTracker**: Tracks which parts of code are executed

### Usage Examples

#### Analyzing Coverage

```python
from coverage_analysis.coverage_analysis import CoverageAnalyzer

# Create a coverage analyzer
analyzer = CoverageAnalyzer(interpreter, "/path/to/output")

# Instrument files for coverage analysis
instrumented_files = analyzer.instrument_files(["/path/to/code.ai"])

# Start tracking execution
analyzer.execution_tracker.start_tracking()

# Run the code
with open(instrumented_files[0], 'r') as f:
    interpreter.execute(f.read())

# Stop tracking
analyzer.execution_tracker.stop_tracking()

# Generate reports
reports = analyzer.generate_reports()
print(f"HTML report: {reports['html']}")
```

#### Generating Coverage Reports

```python
from coverage_analysis.coverage_analysis import CoverageReporter

# Create a coverage reporter
reporter = CoverageReporter(execution_tracker, "/path/to/output")

# Generate a summary report
summary = reporter.generate_summary_report()
print(f"Statement coverage: {summary['statement_coverage']}%")
print(f"Branch coverage: {summary['branch_coverage']}%")

# Generate an HTML report
html_report = reporter.generate_html_report()
print(f"HTML report: {html_report}")
```

#### Saving and Loading Coverage Data

```python
# Save coverage data
data_file = analyzer.save_coverage_data()

# Load coverage data
analyzer.load_coverage_data(data_file)
```

### Advanced Features

- **Statement Coverage**: Tracks which statements are executed
- **Branch Coverage**: Tracks which branches (if/else) are taken
- **Path Coverage**: Tracks which execution paths are followed
- **Function Coverage**: Tracks which functions are called
- **Differential Coverage**: Compares coverage between runs

## Performance Benchmarking

The Performance Benchmarking system measures and compares code performance, focusing on token efficiency.

### Key Components

- **PerformanceBenchmarker**: Main interface for benchmarking
- **BenchmarkSuite**: Collection of related benchmarks
- **BenchmarkRunner**: Executes benchmarks and collects results
- **BenchmarkComparison**: Compares benchmark results

### Usage Examples

#### Creating and Running a Benchmark Suite

```python
from performance_benchmarking.performance_benchmarking import PerformanceBenchmarker

# Create a benchmarker
benchmarker = PerformanceBenchmarker(
    interpreter,
    "/path/to/output",
    iterations=10,
    warmup_iterations=3
)

# Create a benchmark suite
suite = benchmarker.create_suite(
    name="math_suite",
    description="Mathematical operations benchmarks"
)

# Add benchmarks
suite.add_benchmark(
    name="fibonacci",
    code="""
    λ⟨ fibonacci ⟩(n)
        if n <= 1 {
            return n
        }
        return fibonacci(n-1) + fibonacci(n-2)
    
    result ← fibonacci(20)
    """,
    description="Fibonacci sequence calculation"
)

# Run the suite
results = benchmarker.run_suite(suite)

# Generate reports
reports = benchmarker.generate_reports(suite)
print(f"HTML report: {reports['html']}")
```

#### Comparing Benchmark Results

```python
# Save baseline results
benchmarker.save_results(suite, "baseline")

# Later, run again and compare
new_results = benchmarker.run_suite(suite)
comparison = benchmarker.compare_to_baseline(suite, "baseline")

# Get comparison data
comparison_data = comparison.compare()
for benchmark, data in comparison_data.items():
    print(f"{benchmark}: {data['percent_change']}% change in execution time")
```

#### Token Efficiency Analysis

```python
# Get token efficiency metrics
token_metrics = benchmarker.analyze_token_efficiency(suite)
for benchmark, metrics in token_metrics.items():
    print(f"{benchmark}:")
    print(f"  Tokens per operation: {metrics['tokens_per_operation']}")
    print(f"  Token efficiency score: {metrics['efficiency_score']}")
```

### Advanced Features

- **Token Counting**: Measures token usage for efficiency analysis
- **Memory Profiling**: Tracks memory usage during execution
- **Time Series Analysis**: Tracks performance changes over time
- **Comparative Analysis**: Compares different implementations
- **Visualization**: Generates charts and graphs of performance data

## Integration Workflow

The Testing Tools are designed to work together in an integrated workflow:

1. **Record** user interactions with the interpreter
2. **Generate** tests based on recorded sessions
3. **Analyze** code coverage to identify untested areas
4. **Generate** additional tests to improve coverage
5. **Benchmark** performance to identify bottlenecks
6. **Optimize** code based on benchmarking results
7. **Verify** optimizations with regression tests

### Example Integrated Workflow

```python
# Record a session
session = RecordingSession("user_session", interpreter)
session.start_recording()
# ... user interacts with interpreter ...
session.stop_recording()
session.save_recording("/path/to/recording.json")

# Generate tests from the recording
generator = TestGenerator(interpreter)
generator.add_template_from_recording("/path/to/recording.json")
tests = generator.generate_tests("recorded_template", count=10)
generator.save_tests(tests, "/path/to/tests/")

# Analyze coverage
analyzer = CoverageAnalyzer(interpreter, "/path/to/output")
analyzer.instrument_files(["/path/to/code.ai"])
analyzer.run_tests("/path/to/tests/")
reports = analyzer.generate_reports()

# Benchmark performance
benchmarker = PerformanceBenchmarker(interpreter, "/path/to/output")
suite = benchmarker.create_suite("performance_suite")
suite.add_benchmark_from_file("main_benchmark", "/path/to/code.ai")
results = benchmarker.run_suite(suite)
benchmarker.generate_reports(suite)
```

## Best Practices

### Record/Replay

- Record complete sessions that include setup and teardown
- Use descriptive names for recordings
- Organize recordings by feature or functionality
- Verify replays with assertions

### Test Generation

- Create templates for common testing patterns
- Use property-based testing for complex logic
- Generate tests with diverse inputs
- Include edge cases and boundary conditions

### Coverage Analysis

- Aim for high statement and branch coverage (>80%)
- Focus on critical code paths first
- Use coverage data to guide test generation
- Integrate coverage analysis into CI/CD pipelines

### Performance Benchmarking

- Establish baseline benchmarks early
- Run benchmarks on consistent hardware
- Include realistic workloads
- Focus on token efficiency for LLM interactions
- Track performance trends over time

## Conclusion

The Anarchy Inference Testing Tools provide a comprehensive infrastructure for developing, testing, and optimizing Anarchy Inference code. By using these tools together, developers can create robust, efficient, and well-tested applications that leverage the token efficiency advantages of Anarchy Inference.
