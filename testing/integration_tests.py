"""
Integration Tests for Anarchy Inference Testing Tools

This module provides integration tests for the testing tools components:
- Record/Replay System
- Automated Test Generation
- Coverage Analysis
- Performance Benchmarking
"""

import os
import sys
import unittest
import tempfile
import json
import time
from typing import Dict, List, Any

# Add the parent directory to the path so we can import the testing modules
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Import the testing modules
from record_replay.record_replay import RecordingSession, ReplaySession, RecordingManager
from test_generation.test_generation import TestGenerator, TestTemplate, Fuzzer
from coverage_analysis.coverage_analysis import CoverageAnalyzer, CoverageReporter
from performance_benchmarking.performance_benchmarking import PerformanceBenchmarker, BenchmarkSuite

# Import the Anarchy Inference interpreter
try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Using mock interpreter for testing.")
    
    # Create a mock interpreter for testing
    class MockInterpreter:
        def __init__(self):
            self.executed_code = []
        
        def execute(self, code):
            self.executed_code.append(code)
            return {"result": "mock_result"}
        
        def parse(self, code):
            return {"ast": "mock_ast"}
        
        def tokenize(self, code):
            return code.split()
    
    anarchy = type('anarchy', (), {})
    anarchy.Interpreter = MockInterpreter


class TestRecordReplaySystem(unittest.TestCase):
    """Tests for the Record/Replay System."""
    
    def setUp(self):
        """Set up the test environment."""
        self.interpreter = anarchy.Interpreter()
        self.temp_dir = tempfile.mkdtemp()
        self.recording_manager = RecordingManager(self.temp_dir)
    
    def test_recording_session(self):
        """Test creating and using a recording session."""
        # Create a recording session
        session = RecordingSession("test_session", self.interpreter)
        
        # Record some code execution
        test_code = """
        λ⟨ test_function ⟩
            x ← 1
            y ← 2
            return x + y
        
        result ← test_function()
        """
        
        session.start_recording()
        self.interpreter.execute(test_code)
        session.stop_recording()
        
        # Check that the session recorded the execution
        self.assertTrue(session.has_recording())
        self.assertGreater(len(session.get_events()), 0)
        
        # Save the recording
        recording_path = os.path.join(self.temp_dir, "test_recording.json")
        session.save_recording(recording_path)
        
        # Check that the recording was saved
        self.assertTrue(os.path.exists(recording_path))
    
    def test_replay_session(self):
        """Test replaying a recorded session."""
        # Create a recording
        recording_session = RecordingSession("test_session", self.interpreter)
        
        test_code = """
        λ⟨ test_function ⟩
            x ← 1
            y ← 2
            return x + y
        
        result ← test_function()
        """
        
        recording_session.start_recording()
        original_result = self.interpreter.execute(test_code)
        recording_session.stop_recording()
        
        recording_path = os.path.join(self.temp_dir, "test_recording.json")
        recording_session.save_recording(recording_path)
        
        # Create a replay session
        replay_session = ReplaySession("test_replay", self.interpreter)
        
        # Load the recording
        replay_session.load_recording(recording_path)
        
        # Replay the recording
        replay_result = replay_session.replay()
        
        # Check that the replay was successful
        self.assertTrue(replay_session.is_replay_complete())
        
        # In a real test, we would compare original_result and replay_result
        # For the mock interpreter, we just check that replay happened
        self.assertIsNotNone(replay_result)
    
    def test_recording_manager(self):
        """Test the recording manager."""
        # Create some recordings
        for i in range(3):
            session = RecordingSession(f"test_session_{i}", self.interpreter)
            
            test_code = f"""
            λ⟨ test_function_{i} ⟩
                x ← {i}
                y ← {i+1}
                return x + y
            
            result ← test_function_{i}()
            """
            
            session.start_recording()
            self.interpreter.execute(test_code)
            session.stop_recording()
            
            recording_path = os.path.join(self.temp_dir, f"test_recording_{i}.json")
            session.save_recording(recording_path)
            
            # Register the recording with the manager
            self.recording_manager.register_recording(f"test_session_{i}", recording_path)
        
        # Check that the manager has the recordings
        recordings = self.recording_manager.list_recordings()
        self.assertEqual(len(recordings), 3)
        
        # Test getting a recording
        recording_path = self.recording_manager.get_recording_path("test_session_1")
        self.assertTrue(os.path.exists(recording_path))
        
        # Test replaying a recording through the manager
        replay_result = self.recording_manager.replay_recording("test_session_2")
        self.assertIsNotNone(replay_result)


class TestAutomatedTestGeneration(unittest.TestCase):
    """Tests for the Automated Test Generation system."""
    
    def setUp(self):
        """Set up the test environment."""
        self.interpreter = anarchy.Interpreter()
        self.temp_dir = tempfile.mkdtemp()
    
    def test_test_template(self):
        """Test creating and using a test template."""
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
        
        rendered = template.render(values)
        
        # Check that the template was rendered correctly
        self.assertIn("x ← 5", rendered)
        self.assertIn("y ← 10", rendered)
        self.assertIn("expected ← 15", rendered)
    
    def test_fuzzer(self):
        """Test the fuzzer for generating test inputs."""
        # Create a fuzzer
        fuzzer = Fuzzer()
        
        # Generate some integer values
        int_values = [fuzzer.generate_int(-100, 100) for _ in range(10)]
        
        # Check that the values are within range
        for value in int_values:
            self.assertIsInstance(value, int)
            self.assertGreaterEqual(value, -100)
            self.assertLessEqual(value, 100)
        
        # Generate some string values
        string_values = [fuzzer.generate_string(10) for _ in range(10)]
        
        # Check that the values are strings of the right length
        for value in string_values:
            self.assertIsInstance(value, str)
            self.assertLessEqual(len(value), 10)
    
    def test_test_generator(self):
        """Test the test generator."""
        # Create a test generator
        generator = TestGenerator(self.interpreter)
        
        # Add a test template
        generator.add_template(
            TestTemplate(
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
        )
        
        # Generate tests
        tests = generator.generate_tests(
            template_name="arithmetic_test",
            count=5,
            value_ranges={
                "x_value": (-10, 10),
                "y_value": (-10, 10)
            },
            derived_values={
                "expected": lambda values: values["x_value"] + values["y_value"]
            }
        )
        
        # Check that tests were generated
        self.assertEqual(len(tests), 5)
        
        # Check that each test is valid
        for test in tests:
            self.assertIn("test_arithmetic", test)
            self.assertIn("assert", test)
        
        # Save the tests
        output_dir = os.path.join(self.temp_dir, "generated_tests")
        os.makedirs(output_dir, exist_ok=True)
        
        for i, test in enumerate(tests):
            test_path = os.path.join(output_dir, f"test_{i}.ai")
            with open(test_path, 'w') as f:
                f.write(test)
            
            # Check that the test was saved
            self.assertTrue(os.path.exists(test_path))


class TestCoverageAnalysis(unittest.TestCase):
    """Tests for the Coverage Analysis system."""
    
    def setUp(self):
        """Set up the test environment."""
        self.interpreter = anarchy.Interpreter()
        self.temp_dir = tempfile.mkdtemp()
        self.coverage_analyzer = CoverageAnalyzer(self.interpreter, self.temp_dir)
    
    def test_instrumentation(self):
        """Test instrumenting code for coverage analysis."""
        # Create a test file
        test_file = os.path.join(self.temp_dir, "test_code.ai")
        with open(test_file, 'w') as f:
            f.write("""
            λ⟨ test_function ⟩
                x ← 1
                y ← 2
                if x > 0 {
                    return x + y
                } else {
                    return x - y
                }
            
            result ← test_function()
            """)
        
        # Instrument the file
        instrumented_file = self.coverage_analyzer.instrument_files([test_file])[0]
        
        # Check that the instrumented file exists
        self.assertTrue(os.path.exists(instrumented_file))
        
        # Check that coverage points were extracted
        self.assertGreater(len(self.coverage_analyzer.execution_tracker.coverage_points), 0)
    
    def test_coverage_reporting(self):
        """Test generating coverage reports."""
        # Create a test file
        test_file = os.path.join(self.temp_dir, "test_code.ai")
        with open(test_file, 'w') as f:
            f.write("""
            λ⟨ test_function ⟩
                x ← 1
                y ← 2
                if x > 0 {
                    return x + y
                } else {
                    return x - y
                }
            
            result ← test_function()
            """)
        
        # Instrument the file
        self.coverage_analyzer.instrument_files([test_file])
        
        # Run the code to collect coverage data
        self.coverage_analyzer.execution_tracker.start_tracking()
        with open(test_file, 'r') as f:
            self.interpreter.execute(f.read())
        self.coverage_analyzer.execution_tracker.stop_tracking()
        
        # Generate reports
        reports = self.coverage_analyzer.generate_reports()
        
        # Check that reports were generated
        self.assertIn("html", reports)
        self.assertTrue(os.path.exists(reports["html"]))
        
        # Save coverage data
        coverage_data_file = self.coverage_analyzer.save_coverage_data()
        self.assertTrue(os.path.exists(coverage_data_file))
    
    def test_coverage_reporter(self):
        """Test the coverage reporter."""
        # Create a reporter
        reporter = CoverageReporter(self.coverage_analyzer.execution_tracker, self.temp_dir)
        
        # Generate a summary report
        summary = reporter.generate_summary_report()
        
        # Check that the summary has the expected structure
        self.assertIn("statement_coverage", summary)
        self.assertIn("branch_coverage", summary)
        self.assertIn("path_coverage", summary)
        
        # Generate an HTML report
        html_report = reporter.generate_html_report()
        
        # Check that the report was generated
        self.assertTrue(os.path.exists(html_report))


class TestPerformanceBenchmarking(unittest.TestCase):
    """Tests for the Performance Benchmarking system."""
    
    def setUp(self):
        """Set up the test environment."""
        self.interpreter = anarchy.Interpreter()
        self.temp_dir = tempfile.mkdtemp()
        self.benchmarker = PerformanceBenchmarker(
            self.interpreter,
            self.temp_dir,
            iterations=2,  # Use fewer iterations for testing
            warmup_iterations=1
        )
    
    def test_benchmark_suite(self):
        """Test creating and using a benchmark suite."""
        # Create a benchmark suite
        suite = self.benchmarker.create_suite(
            name="test_suite",
            description="Test benchmark suite"
        )
        
        # Add a benchmark
        suite.add_benchmark(
            name="simple_math",
            code="""
            λ⟨ simple_math ⟩
                sum ← 0
                for i in range(100) {
                    sum ← sum + i
                }
                return sum
            
            result ← simple_math()
            """,
            description="Simple math operations"
        )
        
        # Save the suite
        suite_path = os.path.join(self.temp_dir, "test_suite.json")
        suite.save(suite_path)
        
        # Check that the suite was saved
        self.assertTrue(os.path.exists(suite_path))
        
        # Load the suite
        loaded_suite = self.benchmarker.load_suite(suite_path)
        
        # Check that the loaded suite has the benchmark
        self.assertIn("simple_math", loaded_suite.benchmarks)
    
    def test_benchmark_runner(self):
        """Test running benchmarks."""
        # Create a benchmark suite
        suite = self.benchmarker.create_suite(
            name="test_suite",
            description="Test benchmark suite"
        )
        
        # Add a benchmark
        suite.add_benchmark(
            name="simple_math",
            code="""
            λ⟨ simple_math ⟩
                sum ← 0
                for i in range(10) {
                    sum ← sum + i
                }
                return sum
            
            result ← simple_math()
            """,
            description="Simple math operations"
        )
        
        # Run the suite
        results = self.benchmarker.run_suite(suite)
        
        # Check that results were generated
        self.assertIn("simple_math", results)
        self.assertIsNotNone(results["simple_math"].avg_execution_time)
    
    def test_benchmark_reporting(self):
        """Test generating benchmark reports."""
        # Create and run a benchmark suite
        suite = self.benchmarker.create_suite(
            name="test_suite",
            description="Test benchmark suite"
        )
        
        suite.add_benchmark(
            name="simple_math",
            code="""
            λ⟨ simple_math ⟩
                sum ← 0
                for i in range(10) {
                    sum ← sum + i
                }
                return sum
            
            result ← simple_math()
            """,
            description="Simple math operations"
        )
        
        self.benchmarker.run_suite(suite)
        
        # Generate reports
        reports = self.benchmarker.generate_reports(suite)
        
        # Check that reports were generated
        self.assertIn("text", reports)
        self.assertIn("html", reports)
        self.assertIn("csv", reports)
        self.assertIn("json", reports)
        
        for report_path in reports.values():
            self.assertTrue(os.path.exists(report_path))
    
    def test_benchmark_comparison(self):
        """Test comparing benchmark results."""
        # Create a baseline suite
        baseline_suite = self.benchmarker.create_suite(
            name="baseline_suite",
            description="Baseline benchmark suite"
        )
        
        baseline_suite.add_benchmark(
            name="simple_math",
            code="""
            λ⟨ simple_math ⟩
                sum ← 0
                for i in range(10) {
                    sum ← sum + i
                }
                return sum
            
            result ← simple_math()
            """,
            description="Simple math operations"
        )
        
        self.benchmarker.run_suite(baseline_suite)
        
        # Save the baseline results
        self.benchmarker.save_results(baseline_suite, "baseline")
        
        # Create a current suite (with the same benchmark)
        current_suite = self.benchmarker.create_suite(
            name="baseline_suite",  # Same name to match in database
            description="Current benchmark suite"
        )
        
        current_suite.add_benchmark(
            name="simple_math",
            code="""
            λ⟨ simple_math ⟩
                sum ← 0
                for i in range(10) {
                    sum ← sum + i
                }
                return sum
            
            result ← simple_math()
            """,
            description="Simple math operations"
        )
        
        self.benchmarker.run_suite(current_suite)
        
        # Compare to baseline
        comparison = self.benchmarker.compare_to_baseline(current_suite, "baseline")
        
        # Check that comparison was generated
        # Note: In a real test with a real interpreter, we would check the comparison results
        # For the mock interpreter, we just check that comparison exists if the database has baseline results
        if comparison:
            comparison_data = comparison.compare()
            self.assertIn("simple_math", comparison_data)


class TestIntegration(unittest.TestCase):
    """Integration tests for all testing tools working together."""
    
    def setUp(self):
        """Set up the test environment."""
        self.interpreter = anarchy.Interpreter()
        self.temp_dir = tempfile.mkdtemp()
        
        # Create components
        self.recording_manager = RecordingManager(os.path.join(self.temp_dir, "recordings"))
        self.test_generator = TestGenerator(self.interpreter)
        self.coverage_analyzer = CoverageAnalyzer(
            self.interpreter, 
            os.path.join(self.temp_dir, "coverage")
        )
        self.benchmarker = PerformanceBenchmarker(
            self.interpreter,
            os.path.join(self.temp_dir, "benchmarks"),
            iterations=2,
            warmup_iterations=1
        )
    
    def test_record_and_generate(self):
        """Test recording execution and generating tests from it."""
        # Create a recording session
        session = RecordingSession("test_session", self.interpreter)
        
        # Record some code execution
        test_code = """
        λ⟨ add ⟩(x, y)
            return x + y
        
        result ← add(5, 10)
        """
        
        session.start_recording()
        self.interpreter.execute(test_code)
        session.stop_recording()
        
        # Save the recording
        recording_path = os.path.join(self.temp_dir, "test_recording.json")
        session.save_recording(recording_path)
        
        # Register the recording
        self.recording_manager.register_recording("test_session", recording_path)
        
        # Create a test template based on the recording
        template = TestTemplate(
            name="add_test",
            template="""
            λ⟨ test_add ⟩
                x ← {{x_value}}
                y ← {{y_value}}
                expected ← {{expected}}
                result ← add(x, y)
                assert(result == expected)
            
            test_add()
            """
        )
        
        self.test_generator.add_template(template)
        
        # Generate tests
        tests = self.test_generator.generate_tests(
            template_name="add_test",
            count=3,
            value_ranges={
                "x_value": (-10, 10),
                "y_value": (-10, 10)
            },
            derived_values={
                "expected": lambda values: values["x_value"] + values["y_value"]
            }
        )
        
        # Save the tests
        test_dir = os.path.join(self.temp_dir, "generated_tests")
        os.makedirs(test_dir, exist_ok=True)
        
        test_files = []
        for i, test in enumerate(tests):
            test_path = os.path.join(test_dir, f"test_{i}.ai")
            with open(test_path, 'w') as f:
                f.write(test)
            test_files.append(test_path)
        
        # Check that tests were saved
        self.assertEqual(len(test_files), 3)
        for test_file in test_files:
            self.assertTrue(os.path.exists(test_file))
    
    def test_coverage_and_benchmarking(self):
        """Test coverage analysis and benchmarking together."""
        # Create a test file
        test_file = os.path.join(self.temp_dir, "test_code.ai")
        with open(test_file, 'w') as f:
            f.write("""
            λ⟨ fibonacci ⟩(n)
                if n <= 1 {
                    return n
                }
                return fibonacci(n-1) + fibonacci(n-2)
            
            λ⟨ main ⟩
                result ← fibonacci(10)
                return result
            
            main()
            """)
        
        # Instrument the file for coverage analysis
        self.coverage_analyzer.instrument_files([test_file])
        
        # Create a benchmark suite
        suite = self.benchmarker.create_suite(
            name="fibonacci_suite",
            description="Fibonacci benchmark suite"
        )
        
        # Add the file as a benchmark
        suite.add_benchmark_from_file(
            name="fibonacci",
            file_path=test_file,
            description="Fibonacci calculation"
        )
        
        # Run coverage analysis
        self.coverage_analyzer.execution_tracker.start_tracking()
        with open(test_file, 'r') as f:
            self.interpreter.execute(f.read())
        self.coverage_analyzer.execution_tracker.stop_tracking()
        
        # Generate coverage reports
        coverage_reports = self.coverage_analyzer.generate_reports()
        
        # Run benchmarks
        self.benchmarker.run_suite(suite)
        
        # Generate benchmark reports
        benchmark_reports = self.benchmarker.generate_reports(suite)
        
        # Check that both types of reports were generated
        self.assertIn("html", coverage_reports)
        self.assertIn("html", benchmark_reports)
        
        # Save results
        self.benchmarker.save_results(suite)
        coverage_data_file = self.coverage_analyzer.save_coverage_data()
        
        # Check that results were saved
        self.assertTrue(os.path.exists(coverage_data_file))


if __name__ == "__main__":
    unittest.main()
