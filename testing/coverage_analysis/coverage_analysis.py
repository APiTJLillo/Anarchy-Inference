"""
Coverage Analysis for Anarchy Inference

This module provides functionality to analyze code coverage for Anarchy Inference programs,
helping identify untested code paths and improve test quality.
"""

import os
import sys
import json
import time
import hashlib
import re
from typing import Dict, List, Any, Optional, Tuple, Set, Callable, Union
from collections import defaultdict
import html
import matplotlib.pyplot as plt
from matplotlib.colors import LinearSegmentedColormap
import numpy as np

# Add the parent directory to the path so we can import the anarchy module
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import anarchy
except ImportError:
    print("Error: Could not import anarchy module. Make sure it's in the parent directory.")
    sys.exit(1)

class CoveragePoint:
    """Represents a point in the code that can be covered by tests."""
    
    def __init__(self, 
                 file_path: str, 
                 line_number: int, 
                 node_type: str, 
                 node_id: str,
                 parent_id: str = None):
        """Initialize a coverage point.
        
        Args:
            file_path: Path to the source file
            line_number: Line number in the source file
            node_type: Type of AST node
            node_id: Unique identifier for the node
            parent_id: Identifier of the parent node
        """
        self.file_path = file_path
        self.line_number = line_number
        self.node_type = node_type
        self.node_id = node_id
        self.parent_id = parent_id
        self.hit_count = 0
        self.last_hit_time = None
    
    def hit(self):
        """Record a hit on this coverage point."""
        self.hit_count += 1
        self.last_hit_time = time.time()
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert the coverage point to a dictionary."""
        return {
            "file_path": self.file_path,
            "line_number": self.line_number,
            "node_type": self.node_type,
            "node_id": self.node_id,
            "parent_id": self.parent_id,
            "hit_count": self.hit_count,
            "last_hit_time": self.last_hit_time
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'CoveragePoint':
        """Create a coverage point from a dictionary."""
        point = cls(
            file_path=data["file_path"],
            line_number=data["line_number"],
            node_type=data["node_type"],
            node_id=data["node_id"],
            parent_id=data.get("parent_id")
        )
        point.hit_count = data.get("hit_count", 0)
        point.last_hit_time = data.get("last_hit_time")
        return point


class CoverageBranch:
    """Represents a branch in the code that can be covered by tests."""
    
    def __init__(self, 
                 file_path: str, 
                 line_number: int, 
                 branch_id: str,
                 condition: str,
                 parent_id: str = None):
        """Initialize a coverage branch.
        
        Args:
            file_path: Path to the source file
            line_number: Line number in the source file
            branch_id: Unique identifier for the branch
            condition: String representation of the branch condition
            parent_id: Identifier of the parent node
        """
        self.file_path = file_path
        self.line_number = line_number
        self.branch_id = branch_id
        self.condition = condition
        self.parent_id = parent_id
        self.true_hit_count = 0
        self.false_hit_count = 0
        self.last_true_hit_time = None
        self.last_false_hit_time = None
    
    def hit_true(self):
        """Record a hit on the true branch."""
        self.true_hit_count += 1
        self.last_true_hit_time = time.time()
    
    def hit_false(self):
        """Record a hit on the false branch."""
        self.false_hit_count += 1
        self.last_false_hit_time = time.time()
    
    def is_fully_covered(self) -> bool:
        """Check if both branches have been covered."""
        return self.true_hit_count > 0 and self.false_hit_count > 0
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert the coverage branch to a dictionary."""
        return {
            "file_path": self.file_path,
            "line_number": self.line_number,
            "branch_id": self.branch_id,
            "condition": self.condition,
            "parent_id": self.parent_id,
            "true_hit_count": self.true_hit_count,
            "false_hit_count": self.false_hit_count,
            "last_true_hit_time": self.last_true_hit_time,
            "last_false_hit_time": self.last_false_hit_time
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'CoverageBranch':
        """Create a coverage branch from a dictionary."""
        branch = cls(
            file_path=data["file_path"],
            line_number=data["line_number"],
            branch_id=data["branch_id"],
            condition=data["condition"],
            parent_id=data.get("parent_id")
        )
        branch.true_hit_count = data.get("true_hit_count", 0)
        branch.false_hit_count = data.get("false_hit_count", 0)
        branch.last_true_hit_time = data.get("last_true_hit_time")
        branch.last_false_hit_time = data.get("last_false_hit_time")
        return branch


class CoveragePath:
    """Represents an execution path through the code."""
    
    def __init__(self, path_id: str):
        """Initialize a coverage path.
        
        Args:
            path_id: Unique identifier for the path
        """
        self.path_id = path_id
        self.points: List[str] = []  # List of node IDs in the path
        self.branches: List[Tuple[str, bool]] = []  # List of (branch_id, taken) tuples
        self.hit_count = 0
        self.last_hit_time = None
    
    def add_point(self, node_id: str):
        """Add a point to the path.
        
        Args:
            node_id: ID of the node to add
        """
        self.points.append(node_id)
    
    def add_branch(self, branch_id: str, taken: bool):
        """Add a branch to the path.
        
        Args:
            branch_id: ID of the branch
            taken: Whether the branch was taken
        """
        self.branches.append((branch_id, taken))
    
    def hit(self):
        """Record a hit on this path."""
        self.hit_count += 1
        self.last_hit_time = time.time()
    
    def to_dict(self) -> Dict[str, Any]:
        """Convert the coverage path to a dictionary."""
        return {
            "path_id": self.path_id,
            "points": self.points,
            "branches": self.branches,
            "hit_count": self.hit_count,
            "last_hit_time": self.last_hit_time
        }
    
    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> 'CoveragePath':
        """Create a coverage path from a dictionary."""
        path = cls(path_id=data["path_id"])
        path.points = data.get("points", [])
        path.branches = data.get("branches", [])
        path.hit_count = data.get("hit_count", 0)
        path.last_hit_time = data.get("last_hit_time")
        return path


class InstrumentationEngine:
    """Adds tracking code to Anarchy Inference programs for coverage analysis."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter'):
        """Initialize the instrumentation engine.
        
        Args:
            interpreter: The Anarchy Inference interpreter to instrument
        """
        self.interpreter = interpreter
        self.instrumented_files: Dict[str, str] = {}
        self.coverage_points: Dict[str, CoveragePoint] = {}
        self.coverage_branches: Dict[str, CoverageBranch] = {}
        self.ast_cache: Dict[str, Any] = {}
    
    def instrument_file(self, file_path: str) -> str:
        """Instrument a file for coverage analysis.
        
        Args:
            file_path: Path to the file to instrument
            
        Returns:
            Path to the instrumented file
        """
        # Check if already instrumented
        if file_path in self.instrumented_files:
            return self.instrumented_files[file_path]
        
        # Read the file
        with open(file_path, 'r') as f:
            source = f.read()
        
        # Parse the source to get the AST
        ast = self.interpreter.parse(source)
        self.ast_cache[file_path] = ast
        
        # Instrument the source
        instrumented_source = self._instrument_source(source, ast, file_path)
        
        # Write the instrumented source to a new file
        instrumented_path = file_path + ".instrumented"
        with open(instrumented_path, 'w') as f:
            f.write(instrumented_source)
        
        self.instrumented_files[file_path] = instrumented_path
        return instrumented_path
    
    def _instrument_source(self, source: str, ast: Any, file_path: str) -> str:
        """Instrument source code based on its AST.
        
        Args:
            source: Original source code
            ast: Abstract syntax tree of the source
            file_path: Path to the source file
            
        Returns:
            Instrumented source code
        """
        # This is a simplified implementation
        # In a real implementation, this would traverse the AST and insert
        # tracking code at appropriate points
        
        # For demonstration purposes, we'll just add a simple header
        # that loads the coverage tracking library
        header = """# Instrumented for coverage analysis
# Original file: {file_path}
# Instrumentation timestamp: {timestamp}

# Coverage tracking imports would go here
# ...

""".format(file_path=file_path, timestamp=time.time())
        
        # In a real implementation, we would also:
        # 1. Insert tracking calls before each statement
        # 2. Insert branch tracking for conditionals
        # 3. Insert path tracking for function entries/exits
        
        # For now, just return the source with the header
        return header + source
    
    def extract_coverage_points(self, file_path: str) -> Dict[str, CoveragePoint]:
        """Extract coverage points from a file.
        
        Args:
            file_path: Path to the file
            
        Returns:
            Dictionary of coverage points
        """
        # Get the AST
        if file_path not in self.ast_cache:
            with open(file_path, 'r') as f:
                source = f.read()
            ast = self.interpreter.parse(source)
            self.ast_cache[file_path] = ast
        else:
            ast = self.ast_cache[file_path]
        
        # Extract coverage points from the AST
        points = self._extract_points_from_ast(ast, file_path)
        
        # Add to the global coverage points
        self.coverage_points.update(points)
        
        return points
    
    def _extract_points_from_ast(self, ast: Any, file_path: str) -> Dict[str, CoveragePoint]:
        """Extract coverage points from an AST.
        
        Args:
            ast: Abstract syntax tree
            file_path: Path to the source file
            
        Returns:
            Dictionary of coverage points
        """
        # This is a simplified implementation
        # In a real implementation, this would traverse the AST and create
        # coverage points for each node
        
        points = {}
        
        # For demonstration purposes, we'll create some dummy points
        # In a real implementation, these would be based on the actual AST
        
        # Create a point for the file itself
        file_id = hashlib.md5(file_path.encode()).hexdigest()
        file_point = CoveragePoint(
            file_path=file_path,
            line_number=1,
            node_type="File",
            node_id=file_id
        )
        points[file_id] = file_point
        
        # Create some dummy function points
        for i in range(1, 6):
            func_id = f"{file_id}_func_{i}"
            func_point = CoveragePoint(
                file_path=file_path,
                line_number=i * 10,
                node_type="Function",
                node_id=func_id,
                parent_id=file_id
            )
            points[func_id] = func_point
            
            # Create some statement points for each function
            for j in range(1, 4):
                stmt_id = f"{func_id}_stmt_{j}"
                stmt_point = CoveragePoint(
                    file_path=file_path,
                    line_number=i * 10 + j,
                    node_type="Statement",
                    node_id=stmt_id,
                    parent_id=func_id
                )
                points[stmt_id] = stmt_point
        
        return points
    
    def extract_coverage_branches(self, file_path: str) -> Dict[str, CoverageBranch]:
        """Extract coverage branches from a file.
        
        Args:
            file_path: Path to the file
            
        Returns:
            Dictionary of coverage branches
        """
        # Get the AST
        if file_path not in self.ast_cache:
            with open(file_path, 'r') as f:
                source = f.read()
            ast = self.interpreter.parse(source)
            self.ast_cache[file_path] = ast
        else:
            ast = self.ast_cache[file_path]
        
        # Extract coverage branches from the AST
        branches = self._extract_branches_from_ast(ast, file_path)
        
        # Add to the global coverage branches
        self.coverage_branches.update(branches)
        
        return branches
    
    def _extract_branches_from_ast(self, ast: Any, file_path: str) -> Dict[str, CoverageBranch]:
        """Extract coverage branches from an AST.
        
        Args:
            ast: Abstract syntax tree
            file_path: Path to the source file
            
        Returns:
            Dictionary of coverage branches
        """
        # This is a simplified implementation
        # In a real implementation, this would traverse the AST and create
        # coverage branches for each conditional
        
        branches = {}
        
        # For demonstration purposes, we'll create some dummy branches
        # In a real implementation, these would be based on the actual AST
        
        # Create a branch for each function
        file_id = hashlib.md5(file_path.encode()).hexdigest()
        
        for i in range(1, 6):
            func_id = f"{file_id}_func_{i}"
            
            # Create some conditional branches
            for j in range(1, 3):
                branch_id = f"{func_id}_branch_{j}"
                condition = f"x > {j * 10}"
                branch = CoverageBranch(
                    file_path=file_path,
                    line_number=i * 10 + j * 2,
                    branch_id=branch_id,
                    condition=condition,
                    parent_id=func_id
                )
                branches[branch_id] = branch
        
        return branches


class ExecutionTracker:
    """Records which parts of code are executed during tests."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter'):
        """Initialize the execution tracker.
        
        Args:
            interpreter: The Anarchy Inference interpreter to track
        """
        self.interpreter = interpreter
        self.coverage_points: Dict[str, CoveragePoint] = {}
        self.coverage_branches: Dict[str, CoverageBranch] = {}
        self.coverage_paths: Dict[str, CoveragePath] = {}
        self.current_path: Optional[CoveragePath] = None
        self._install_hooks()
    
    def _install_hooks(self):
        """Install hooks into the interpreter to track execution."""
        # This would hook into the interpreter's execution pipeline
        # Implementation depends on the specific interpreter API
        pass
    
    def start_tracking(self):
        """Start tracking execution."""
        # Create a new path for this execution
        path_id = f"path_{time.time()}_{hash(self)}"
        self.current_path = CoveragePath(path_id)
        self.coverage_paths[path_id] = self.current_path
    
    def stop_tracking(self):
        """Stop tracking execution."""
        if self.current_path:
            self.current_path.hit()
            self.current_path = None
    
    def record_point_hit(self, node_id: str):
        """Record a hit on a coverage point.
        
        Args:
            node_id: ID of the node that was hit
        """
        if node_id in self.coverage_points:
            self.coverage_points[node_id].hit()
        
        if self.current_path:
            self.current_path.add_point(node_id)
    
    def record_branch_hit(self, branch_id: str, taken: bool):
        """Record a hit on a coverage branch.
        
        Args:
            branch_id: ID of the branch
            taken: Whether the branch was taken
        """
        if branch_id in self.coverage_branches:
            if taken:
                self.coverage_branches[branch_id].hit_true()
            else:
                self.coverage_branches[branch_id].hit_false()
        
        if self.current_path:
            self.current_path.add_branch(branch_id, taken)
    
    def merge_coverage_data(self, other: 'ExecutionTracker'):
        """Merge coverage data from another tracker.
        
        Args:
            other: Another execution tracker
        """
        # Merge coverage points
        for node_id, point in other.coverage_points.items():
            if node_id in self.coverage_points:
                self.coverage_points[node_id].hit_count += point.hit_count
                if point.last_hit_time and (not self.coverage_points[node_id].last_hit_time or 
                                           point.last_hit_time > self.coverage_points[node_id].last_hit_time):
                    self.coverage_points[node_id].last_hit_time = point.last_hit_time
            else:
                self.coverage_points[node_id] = point
        
        # Merge coverage branches
        for branch_id, branch in other.coverage_branches.items():
            if branch_id in self.coverage_branches:
                self.coverage_branches[branch_id].true_hit_count += branch.true_hit_count
                self.coverage_branches[branch_id].false_hit_count += branch.false_hit_count
                
                if branch.last_true_hit_time and (not self.coverage_branches[branch_id].last_true_hit_time or 
                                                branch.last_true_hit_time > self.coverage_branches[branch_id].last_true_hit_time):
                    self.coverage_branches[branch_id].last_true_hit_time = branch.last_true_hit_time
                
                if branch.last_false_hit_time and (not self.coverage_branches[branch_id].last_false_hit_time or 
                                                 branch.last_false_hit_time > self.coverage_branches[branch_id].last_false_hit_time):
                    self.coverage_branches[branch_id].last_false_hit_time = branch.last_false_hit_time
            else:
                self.coverage_branches[branch_id] = branch
        
        # Merge coverage paths
        for path_id, path in other.coverage_paths.items():
            if path_id in self.coverage_paths:
                self.coverage_paths[path_id].hit_count += path.hit_count
                if path.last_hit_time and (not self.coverage_paths[path_id].last_hit_time or 
                                          path.last_hit_time > self.coverage_paths[path_id].last_hit_time):
                    self.coverage_paths[path_id].last_hit_time = path.last_hit_time
            else:
                self.coverage_paths[path_id] = path
    
    def save_coverage_data(self, output_file: str):
        """Save coverage data to a file.
        
        Args:
            output_file: Path to the output file
        """
        data = {
            "points": {node_id: point.to_dict() for node_id, point in self.coverage_points.items()},
            "branches": {branch_id: branch.to_dict() for branch_id, branch in self.coverage_branches.items()},
            "paths": {path_id: path.to_dict() for path_id, path in self.coverage_paths.items()},
            "timestamp": time.time()
        }
        
        with open(output_file, 'w') as f:
            json.dump(data, f, indent=2)
    
    def load_coverage_data(self, input_file: str):
        """Load coverage data from a file.
        
        Args:
            input_file: Path to the input file
        """
        with open(input_file, 'r') as f:
            data = json.load(f)
        
        self.coverage_points = {node_id: CoveragePoint.from_dict(point_data) 
                               for node_id, point_data in data.get("points", {}).items()}
        
        self.coverage_branches = {branch_id: CoverageBranch.from_dict(branch_data) 
                                 for branch_id, branch_data in data.get("branches", {}).items()}
        
        self.coverage_paths = {path_id: CoveragePath.from_dict(path_data) 
                              for path_id, path_data in data.get("paths", {}).items()}


class CoverageReporter:
    """Generates human-readable reports of coverage data."""
    
    def __init__(self, tracker: ExecutionTracker, output_dir: str = None):
        """Initialize the coverage reporter.
        
        Args:
            tracker: The execution tracker with coverage data
            output_dir: Directory to save reports
        """
        self.tracker = tracker
        self.output_dir = output_dir or os.path.join(
            os.path.dirname(os.path.dirname(__file__)), 
            "coverage_reports"
        )
        
        # Create the output directory if it doesn't exist
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)
    
    def generate_summary_report(self) -> Dict[str, Any]:
        """Generate a summary of coverage data.
        
        Returns:
            Dictionary with coverage summary
        """
        # Calculate statement coverage
        total_points = len(self.tracker.coverage_points)
        covered_points = sum(1 for point in self.tracker.coverage_points.values() if point.hit_count > 0)
        statement_coverage = covered_points / total_points if total_points > 0 else 0
        
        # Calculate branch coverage
        total_branches = len(self.tracker.coverage_branches)
        fully_covered_branches = sum(1 for branch in self.tracker.coverage_branches.values() 
                                    if branch.true_hit_count > 0 and branch.false_hit_count > 0)
        partially_covered_branches = sum(1 for branch in self.tracker.coverage_branches.values() 
                                        if (branch.true_hit_count > 0 or branch.false_hit_count > 0) and 
                                           not (branch.true_hit_count > 0 and branch.false_hit_count > 0))
        branch_coverage = fully_covered_branches / total_branches if total_branches > 0 else 0
        
        # Calculate path coverage
        # This is a simplified calculation; in reality, path coverage is more complex
        total_paths = len(self.tracker.coverage_paths)
        covered_paths = sum(1 for path in self.tracker.coverage_paths.values() if path.hit_count > 0)
        path_coverage = covered_paths / total_paths if total_paths > 0 else 0
        
        # Create summary
        summary = {
            "statement_coverage": {
                "total": total_points,
                "covered": covered_points,
                "percentage": statement_coverage * 100
            },
            "branch_coverage": {
                "total": total_branches,
                "fully_covered": fully_covered_branches,
                "partially_covered": partially_covered_branches,
                "percentage": branch_coverage * 100
            },
            "path_coverage": {
                "total": total_paths,
                "covered": covered_paths,
                "percentage": path_coverage * 100
            },
            "timestamp": time.time()
        }
        
        return summary
    
    def generate_file_coverage_report(self, file_path: str) -> Dict[str, Any]:
        """Generate a coverage report for a specific file.
        
        Args:
            file_path: Path to the file
            
        Returns:
            Dictionary with file coverage data
        """
        # Get points and branches for this file
        file_points = {node_id: point for node_id, point in self.tracker.coverage_points.items() 
                      if point.file_path == file_path}
        
        file_branches = {branch_id: branch for branch_id, branch in self.tracker.coverage_branches.items() 
                        if branch.file_path == file_path}
        
        # Calculate line coverage
        line_coverage = defaultdict(lambda: {"hit_count": 0, "type": "statement"})
        
        for point in file_points.values():
            if point.hit_count > 0:
                line_coverage[point.line_number]["hit_count"] += point.hit_count
        
        for branch in file_branches.values():
            line_coverage[branch.line_number]["type"] = "branch"
            if branch.true_hit_count > 0:
                line_coverage[branch.line_number]["hit_count"] += branch.true_hit_count
            if branch.false_hit_count > 0:
                line_coverage[branch.line_number]["hit_count"] += branch.false_hit_count
        
        # Read the file content
        try:
            with open(file_path, 'r') as f:
                lines = f.readlines()
        except FileNotFoundError:
            lines = []
        
        # Create line-by-line coverage data
        line_data = []
        for i, line in enumerate(lines, 1):
            coverage_info = line_coverage.get(i, {"hit_count": 0, "type": "statement"})
            line_data.append({
                "line_number": i,
                "content": line.rstrip(),
                "hit_count": coverage_info["hit_count"],
                "type": coverage_info["type"],
                "covered": coverage_info["hit_count"] > 0
            })
        
        # Calculate coverage percentages
        total_lines = len(line_data)
        covered_lines = sum(1 for line in line_data if line["covered"])
        line_coverage_percentage = covered_lines / total_lines if total_lines > 0 else 0
        
        total_branches = len(file_branches)
        fully_covered_branches = sum(1 for branch in file_branches.values() 
                                    if branch.true_hit_count > 0 and branch.false_hit_count > 0)
        branch_coverage_percentage = fully_covered_branches / total_branches if total_branches > 0 else 0
        
        # Create report
        report = {
            "file_path": file_path,
            "line_coverage": {
                "total": total_lines,
                "covered": covered_lines,
                "percentage": line_coverage_percentage * 100
            },
            "branch_coverage": {
                "total": total_branches,
                "covered": fully_covered_branches,
                "percentage": branch_coverage_percentage * 100
            },
            "line_data": line_data,
            "timestamp": time.time()
        }
        
        return report
    
    def generate_html_report(self, title: str = "Anarchy Inference Coverage Report") -> str:
        """Generate an HTML coverage report.
        
        Args:
            title: Title for the report
            
        Returns:
            Path to the generated HTML report
        """
        # Get summary data
        summary = self.generate_summary_report()
        
        # Get file coverage data
        file_paths = set()
        for point in self.tracker.coverage_points.values():
            file_paths.add(point.file_path)
        
        file_reports = {}
        for file_path in file_paths:
            file_reports[file_path] = self.generate_file_coverage_report(file_path)
        
        # Create HTML content
        html_content = f"""<!DOCTYPE html>
<html>
<head>
    <title>{html.escape(title)}</title>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 0;
            padding: 20px;
            color: #333;
        }}
        h1, h2, h3 {{
            color: #2c3e50;
        }}
        .summary {{
            background-color: #f8f9fa;
            border-radius: 5px;
            padding: 15px;
            margin-bottom: 20px;
        }}
        .progress {{
            height: 20px;
            background-color: #e9ecef;
            border-radius: 5px;
            margin-bottom: 10px;
        }}
        .progress-bar {{
            height: 100%;
            border-radius: 5px;
            background-color: #4caf50;
            text-align: center;
            color: white;
            line-height: 20px;
        }}
        .file-list {{
            list-style-type: none;
            padding: 0;
        }}
        .file-item {{
            padding: 10px;
            border-bottom: 1px solid #ddd;
        }}
        .file-item:hover {{
            background-color: #f5f5f5;
        }}
        .file-link {{
            text-decoration: none;
            color: #3498db;
        }}
        .coverage-table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 10px;
        }}
        .coverage-table th, .coverage-table td {{
            padding: 8px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        .coverage-table th {{
            background-color: #f2f2f2;
        }}
        .line-number {{
            color: #999;
            text-align: right;
            padding-right: 10px;
            user-select: none;
        }}
        .line-content {{
            white-space: pre;
            font-family: monospace;
        }}
        .covered {{
            background-color: #dff0d8;
        }}
        .not-covered {{
            background-color: #f2dede;
        }}
        .branch {{
            background-color: #fcf8e3;
        }}
        .branch.covered {{
            background-color: #dff0d8;
        }}
        .hit-count {{
            color: #999;
            text-align: right;
            padding-right: 10px;
        }}
        .timestamp {{
            color: #999;
            font-size: 0.8em;
            margin-top: 20px;
        }}
    </style>
</head>
<body>
    <h1>{html.escape(title)}</h1>
    
    <div class="summary">
        <h2>Coverage Summary</h2>
        
        <h3>Statement Coverage: {summary['statement_coverage']['percentage']:.2f}%</h3>
        <div class="progress">
            <div class="progress-bar" style="width: {summary['statement_coverage']['percentage']}%">
                {summary['statement_coverage']['covered']} / {summary['statement_coverage']['total']}
            </div>
        </div>
        
        <h3>Branch Coverage: {summary['branch_coverage']['percentage']:.2f}%</h3>
        <div class="progress">
            <div class="progress-bar" style="width: {summary['branch_coverage']['percentage']}%">
                {summary['branch_coverage']['fully_covered']} / {summary['branch_coverage']['total']}
            </div>
        </div>
        
        <h3>Path Coverage: {summary['path_coverage']['percentage']:.2f}%</h3>
        <div class="progress">
            <div class="progress-bar" style="width: {summary['path_coverage']['percentage']}%">
                {summary['path_coverage']['covered']} / {summary['path_coverage']['total']}
            </div>
        </div>
    </div>
    
    <h2>Files</h2>
    <ul class="file-list">
"""
        
        # Add file list
        for file_path, report in file_reports.items():
            file_name = os.path.basename(file_path)
            line_percentage = report['line_coverage']['percentage']
            branch_percentage = report['branch_coverage']['percentage']
            
            html_content += f"""
        <li class="file-item">
            <a href="#{html.escape(file_path)}" class="file-link">{html.escape(file_name)}</a>
            <div class="progress">
                <div class="progress-bar" style="width: {line_percentage}%">
                    Line: {line_percentage:.2f}%
                </div>
            </div>
            <div class="progress">
                <div class="progress-bar" style="width: {branch_percentage}%">
                    Branch: {branch_percentage:.2f}%
                </div>
            </div>
        </li>
"""
        
        html_content += """
    </ul>
    
    <h2>File Details</h2>
"""
        
        # Add file details
        for file_path, report in file_reports.items():
            file_name = os.path.basename(file_path)
            line_percentage = report['line_coverage']['percentage']
            branch_percentage = report['branch_coverage']['percentage']
            
            html_content += f"""
    <div id="{html.escape(file_path)}">
        <h3>{html.escape(file_name)}</h3>
        <p>Line Coverage: {line_percentage:.2f}% ({report['line_coverage']['covered']} / {report['line_coverage']['total']})</p>
        <p>Branch Coverage: {branch_percentage:.2f}% ({report['branch_coverage']['covered']} / {report['branch_coverage']['total']})</p>
        
        <table class="coverage-table">
            <tr>
                <th>Line</th>
                <th>Hits</th>
                <th>Code</th>
            </tr>
"""
            
            # Add line data
            for line in report['line_data']:
                line_class = ""
                if line['type'] == 'branch':
                    line_class = "branch"
                    if line['covered']:
                        line_class += " covered"
                else:
                    if line['covered']:
                        line_class = "covered"
                    else:
                        line_class = "not-covered"
                
                html_content += f"""
            <tr class="{line_class}">
                <td class="line-number">{line['line_number']}</td>
                <td class="hit-count">{line['hit_count']}</td>
                <td class="line-content">{html.escape(line['content'])}</td>
            </tr>
"""
            
            html_content += """
        </table>
    </div>
"""
        
        # Add timestamp
        timestamp = time.strftime('%Y-%m-%d %H:%M:%S', time.localtime(time.time()))
        html_content += f"""
    <div class="timestamp">
        Generated on: {timestamp}
    </div>
</body>
</html>
"""
        
        # Write HTML to file
        output_path = os.path.join(self.output_dir, "coverage_report.html")
        with open(output_path, 'w') as f:
            f.write(html_content)
        
        return output_path
    
    def generate_visualization(self, output_file: str = None) -> str:
        """Generate a visualization of coverage data.
        
        Args:
            output_file: Path to the output file
            
        Returns:
            Path to the generated visualization
        """
        if output_file is None:
            output_file = os.path.join(self.output_dir, "coverage_visualization.png")
        
        # Get summary data
        summary = self.generate_summary_report()
        
        # Create figure
        fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(12, 6))
        
        # Coverage percentages
        categories = ['Statement', 'Branch', 'Path']
        percentages = [
            summary['statement_coverage']['percentage'],
            summary['branch_coverage']['percentage'],
            summary['path_coverage']['percentage']
        ]
        
        # Bar chart
        ax1.bar(categories, percentages, color=['#4caf50', '#2196f3', '#ff9800'])
        ax1.set_ylim(0, 100)
        ax1.set_ylabel('Coverage (%)')
        ax1.set_title('Coverage by Type')
        
        # Add percentage labels
        for i, v in enumerate(percentages):
            ax1.text(i, v + 2, f"{v:.1f}%", ha='center')
        
        # Get file coverage data
        file_paths = set()
        for point in self.tracker.coverage_points.values():
            file_paths.add(point.file_path)
        
        file_reports = {}
        for file_path in file_paths:
            file_reports[file_path] = self.generate_file_coverage_report(file_path)
        
        # File coverage heatmap
        file_names = [os.path.basename(path) for path in file_paths]
        file_line_coverage = [report['line_coverage']['percentage'] for report in file_reports.values()]
        file_branch_coverage = [report['branch_coverage']['percentage'] for report in file_reports.values()]
        
        # Sort by line coverage
        sorted_indices = np.argsort(file_line_coverage)
        file_names = [file_names[i] for i in sorted_indices]
        file_line_coverage = [file_line_coverage[i] for i in sorted_indices]
        file_branch_coverage = [file_branch_coverage[i] for i in sorted_indices]
        
        # Create heatmap data
        heatmap_data = np.array([file_line_coverage, file_branch_coverage])
        
        # Create custom colormap
        cmap = LinearSegmentedColormap.from_list('coverage_cmap', ['#f2dede', '#dff0d8'])
        
        # Plot heatmap
        im = ax2.imshow(heatmap_data, cmap=cmap, aspect='auto', vmin=0, vmax=100)
        
        # Add labels
        ax2.set_yticks([0, 1])
        ax2.set_yticklabels(['Line', 'Branch'])
        
        if len(file_names) <= 10:
            ax2.set_xticks(range(len(file_names)))
            ax2.set_xticklabels(file_names, rotation=45, ha='right')
        else:
            # If too many files, show only some labels
            step = len(file_names) // 10
            ax2.set_xticks(range(0, len(file_names), step))
            ax2.set_xticklabels([file_names[i] for i in range(0, len(file_names), step)], rotation=45, ha='right')
        
        ax2.set_title('Coverage by File')
        
        # Add colorbar
        cbar = fig.colorbar(im, ax=ax2)
        cbar.set_label('Coverage (%)')
        
        # Adjust layout
        plt.tight_layout()
        
        # Save figure
        plt.savefig(output_file, dpi=300, bbox_inches='tight')
        
        return output_file


class CoverageDatabase:
    """Stores historical coverage data."""
    
    def __init__(self, db_file: str = None):
        """Initialize the coverage database.
        
        Args:
            db_file: Path to the database file
        """
        self.db_file = db_file or os.path.join(
            os.path.dirname(os.path.dirname(__file__)), 
            "coverage_db.json"
        )
        self.history = []
        
        # Load existing data if available
        if os.path.exists(self.db_file):
            self.load()
    
    def add_entry(self, summary: Dict[str, Any], commit_id: str = None):
        """Add a coverage summary to the database.
        
        Args:
            summary: Coverage summary data
            commit_id: Optional Git commit ID
        """
        entry = {
            "timestamp": time.time(),
            "commit_id": commit_id,
            "summary": summary
        }
        
        self.history.append(entry)
        self.save()
    
    def get_trend(self, days: int = 30) -> List[Dict[str, Any]]:
        """Get coverage trend data for a specified period.
        
        Args:
            days: Number of days to include
            
        Returns:
            List of coverage entries
        """
        cutoff_time = time.time() - (days * 24 * 60 * 60)
        return [entry for entry in self.history if entry["timestamp"] >= cutoff_time]
    
    def save(self):
        """Save the database to disk."""
        with open(self.db_file, 'w') as f:
            json.dump({"history": self.history}, f, indent=2)
    
    def load(self):
        """Load the database from disk."""
        try:
            with open(self.db_file, 'r') as f:
                data = json.load(f)
                self.history = data.get("history", [])
        except (json.JSONDecodeError, FileNotFoundError):
            self.history = []


class UncoveredCodeAnalyzer:
    """Identifies code not covered by tests."""
    
    def __init__(self, tracker: ExecutionTracker):
        """Initialize the uncovered code analyzer.
        
        Args:
            tracker: The execution tracker with coverage data
        """
        self.tracker = tracker
    
    def find_uncovered_points(self) -> Dict[str, List[CoveragePoint]]:
        """Find uncovered points grouped by file.
        
        Returns:
            Dictionary mapping file paths to lists of uncovered points
        """
        uncovered = defaultdict(list)
        
        for point in self.tracker.coverage_points.values():
            if point.hit_count == 0:
                uncovered[point.file_path].append(point)
        
        return dict(uncovered)
    
    def find_uncovered_branches(self) -> Dict[str, List[CoverageBranch]]:
        """Find uncovered branches grouped by file.
        
        Returns:
            Dictionary mapping file paths to lists of uncovered branches
        """
        uncovered = defaultdict(list)
        
        for branch in self.tracker.coverage_branches.values():
            if branch.true_hit_count == 0 or branch.false_hit_count == 0:
                uncovered[branch.file_path].append(branch)
        
        return dict(uncovered)
    
    def generate_report(self) -> Dict[str, Any]:
        """Generate a report of uncovered code.
        
        Returns:
            Dictionary with uncovered code report
        """
        uncovered_points = self.find_uncovered_points()
        uncovered_branches = self.find_uncovered_branches()
        
        # Group by file
        files = set(uncovered_points.keys()) | set(uncovered_branches.keys())
        
        report = {
            "files": {},
            "total_uncovered_points": sum(len(points) for points in uncovered_points.values()),
            "total_uncovered_branches": sum(len(branches) for branches in uncovered_branches.values()),
            "timestamp": time.time()
        }
        
        for file_path in files:
            file_report = {
                "uncovered_points": [],
                "uncovered_branches": []
            }
            
            # Add uncovered points
            for point in uncovered_points.get(file_path, []):
                file_report["uncovered_points"].append({
                    "line_number": point.line_number,
                    "node_type": point.node_type,
                    "node_id": point.node_id
                })
            
            # Add uncovered branches
            for branch in uncovered_branches.get(file_path, []):
                file_report["uncovered_branches"].append({
                    "line_number": branch.line_number,
                    "condition": branch.condition,
                    "branch_id": branch.branch_id,
                    "true_covered": branch.true_hit_count > 0,
                    "false_covered": branch.false_hit_count > 0
                })
            
            report["files"][file_path] = file_report
        
        return report
    
    def suggest_test_improvements(self) -> Dict[str, List[str]]:
        """Suggest improvements to increase coverage.
        
        Returns:
            Dictionary mapping files to lists of suggestions
        """
        uncovered_points = self.find_uncovered_points()
        uncovered_branches = self.find_uncovered_branches()
        
        suggestions = defaultdict(list)
        
        # Analyze uncovered points
        for file_path, points in uncovered_points.items():
            if points:
                suggestions[file_path].append(f"Add tests for {len(points)} uncovered statements")
                
                # Group by node type
                by_type = defaultdict(list)
                for point in points:
                    by_type[point.node_type].append(point)
                
                for node_type, type_points in by_type.items():
                    suggestions[file_path].append(f"Focus on {len(type_points)} uncovered {node_type.lower()} nodes")
        
        # Analyze uncovered branches
        for file_path, branches in uncovered_branches.items():
            if branches:
                # Count branches missing true case
                missing_true = sum(1 for branch in branches if branch.true_hit_count == 0)
                
                # Count branches missing false case
                missing_false = sum(1 for branch in branches if branch.false_hit_count == 0)
                
                if missing_true > 0:
                    suggestions[file_path].append(f"Add tests for {missing_true} uncovered true branches")
                
                if missing_false > 0:
                    suggestions[file_path].append(f"Add tests for {missing_false} uncovered false branches")
        
        return dict(suggestions)


class CoverageAnalyzer:
    """Main class for analyzing code coverage."""
    
    def __init__(self, interpreter: 'anarchy.Interpreter', output_dir: str = None):
        """Initialize the coverage analyzer.
        
        Args:
            interpreter: The Anarchy Inference interpreter to use
            output_dir: Directory to save coverage data and reports
        """
        self.interpreter = interpreter
        self.output_dir = output_dir or os.path.join(
            os.path.dirname(os.path.dirname(__file__)), 
            "coverage"
        )
        
        # Create the output directory if it doesn't exist
        if not os.path.exists(self.output_dir):
            os.makedirs(self.output_dir)
        
        # Initialize components
        self.instrumentation_engine = InstrumentationEngine(interpreter)
        self.execution_tracker = ExecutionTracker(interpreter)
        self.reporter = CoverageReporter(self.execution_tracker, self.output_dir)
        self.database = CoverageDatabase(os.path.join(self.output_dir, "coverage_db.json"))
        self.uncovered_analyzer = UncoveredCodeAnalyzer(self.execution_tracker)
    
    def instrument_files(self, file_paths: List[str]) -> List[str]:
        """Instrument files for coverage analysis.
        
        Args:
            file_paths: Paths to the files to instrument
            
        Returns:
            Paths to the instrumented files
        """
        instrumented_paths = []
        
        for file_path in file_paths:
            instrumented_path = self.instrumentation_engine.instrument_file(file_path)
            instrumented_paths.append(instrumented_path)
            
            # Extract coverage points and branches
            self.instrumentation_engine.extract_coverage_points(file_path)
            self.instrumentation_engine.extract_coverage_branches(file_path)
        
        # Update the tracker with the coverage points and branches
        self.execution_tracker.coverage_points.update(self.instrumentation_engine.coverage_points)
        self.execution_tracker.coverage_branches.update(self.instrumentation_engine.coverage_branches)
        
        return instrumented_paths
    
    def run_tests(self, test_files: List[str]) -> Dict[str, bool]:
        """Run tests and collect coverage data.
        
        Args:
            test_files: Paths to the test files
            
        Returns:
            Dictionary mapping test files to success status
        """
        results = {}
        
        for test_file in test_files:
            # Start tracking
            self.execution_tracker.start_tracking()
            
            try:
                # Read the test file
                with open(test_file, 'r') as f:
                    test_code = f.read()
                
                # Execute the test
                self.interpreter.execute(test_code)
                results[test_file] = True
            except Exception as e:
                print(f"Error running test {test_file}: {e}")
                results[test_file] = False
            finally:
                # Stop tracking
                self.execution_tracker.stop_tracking()
        
        return results
    
    def generate_reports(self) -> Dict[str, str]:
        """Generate coverage reports.
        
        Returns:
            Dictionary mapping report types to file paths
        """
        reports = {}
        
        # Generate HTML report
        html_report = self.reporter.generate_html_report()
        reports["html"] = html_report
        
        # Generate visualization
        visualization = self.reporter.generate_visualization()
        reports["visualization"] = visualization
        
        # Generate uncovered code report
        uncovered_report = self.uncovered_analyzer.generate_report()
        uncovered_report_path = os.path.join(self.output_dir, "uncovered_report.json")
        with open(uncovered_report_path, 'w') as f:
            json.dump(uncovered_report, f, indent=2)
        reports["uncovered"] = uncovered_report_path
        
        # Generate suggestions
        suggestions = self.uncovered_analyzer.suggest_test_improvements()
        suggestions_path = os.path.join(self.output_dir, "test_suggestions.json")
        with open(suggestions_path, 'w') as f:
            json.dump(suggestions, f, indent=2)
        reports["suggestions"] = suggestions_path
        
        # Add to database
        summary = self.reporter.generate_summary_report()
        self.database.add_entry(summary)
        
        return reports
    
    def save_coverage_data(self) -> str:
        """Save coverage data to a file.
        
        Returns:
            Path to the saved file
        """
        output_file = os.path.join(self.output_dir, "coverage_data.json")
        self.execution_tracker.save_coverage_data(output_file)
        return output_file
    
    def load_coverage_data(self, input_file: str):
        """Load coverage data from a file.
        
        Args:
            input_file: Path to the input file
        """
        self.execution_tracker.load_coverage_data(input_file)
    
    def merge_coverage_data(self, other_file: str) -> str:
        """Merge coverage data from another file.
        
        Args:
            other_file: Path to the other coverage data file
            
        Returns:
            Path to the merged file
        """
        # Create a temporary tracker
        temp_tracker = ExecutionTracker(self.interpreter)
        
        # Load the other file
        temp_tracker.load_coverage_data(other_file)
        
        # Merge with current tracker
        self.execution_tracker.merge_coverage_data(temp_tracker)
        
        # Save the merged data
        return self.save_coverage_data()
    
    def analyze_project(self, project_dir: str) -> Dict[str, Any]:
        """Analyze coverage for an entire project.
        
        Args:
            project_dir: Path to the project directory
            
        Returns:
            Dictionary with analysis results
        """
        # Find all source files
        source_files = []
        for root, _, files in os.walk(project_dir):
            for file in files:
                if file.endswith(".ai"):  # Anarchy Inference files
                    source_files.append(os.path.join(root, file))
        
        # Find all test files
        test_files = []
        for root, _, files in os.walk(os.path.join(project_dir, "tests")):
            for file in files:
                if file.endswith(".ai"):  # Anarchy Inference test files
                    test_files.append(os.path.join(root, file))
        
        # Instrument source files
        instrumented_files = self.instrument_files(source_files)
        
        # Run tests
        test_results = self.run_tests(test_files)
        
        # Generate reports
        reports = self.generate_reports()
        
        # Save coverage data
        coverage_data_file = self.save_coverage_data()
        
        # Return analysis results
        return {
            "source_files": source_files,
            "test_files": test_files,
            "instrumented_files": instrumented_files,
            "test_results": test_results,
            "reports": reports,
            "coverage_data_file": coverage_data_file
        }
