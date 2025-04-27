"""
Benchmark Configuration System for Anarchy Inference

This module provides a flexible configuration system for benchmarks,
allowing customization of benchmark parameters and profiles.
"""

import os
import sys
import json
import yaml
from typing import Dict, List, Any, Optional, Union
from dataclasses import dataclass, field

# Add the parent directory to the path so we can import the performance_benchmarking module
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

@dataclass
class BenchmarkConfig:
    """Configuration for a single benchmark."""
    
    name: str
    description: str = ""
    enabled: bool = True
    iterations: int = 5
    warmup_iterations: int = 2
    parameters: Dict[str, Any] = field(default_factory=dict)
    tags: List[str] = field(default_factory=list)


@dataclass
class CategoryConfig:
    """Configuration for a benchmark category."""
    
    name: str
    description: str = ""
    enabled: bool = True
    benchmarks: Dict[str, BenchmarkConfig] = field(default_factory=dict)


@dataclass
class ProfileConfig:
    """Configuration for a benchmark profile."""
    
    name: str
    description: str = ""
    iterations: int = 5
    warmup_iterations: int = 2
    gc_between_runs: bool = True
    categories: Dict[str, CategoryConfig] = field(default_factory=dict)
    languages: List[str] = field(default_factory=list)
    output_formats: List[str] = field(default_factory=list)
    output_dir: str = "benchmark_reports"


class ConfigurationManager:
    """Manages benchmark configurations."""
    
    def __init__(self, config_dir: str = None):
        """Initialize the configuration manager.
        
        Args:
            config_dir: Directory containing configuration files
        """
        self.config_dir = config_dir or os.path.join(
            os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
            "config"
        )
        
        # Create the config directory if it doesn't exist
        if not os.path.exists(self.config_dir):
            os.makedirs(self.config_dir)
        
        self.profiles: Dict[str, ProfileConfig] = {}
        self.default_profile: Optional[str] = None
    
    def load_profiles(self) -> Dict[str, ProfileConfig]:
        """Load all profile configurations from the config directory.
        
        Returns:
            Dictionary mapping profile names to configurations
        """
        self.profiles = {}
        
        # Load all YAML and JSON files in the config directory
        for filename in os.listdir(self.config_dir):
            if filename.endswith(('.yaml', '.yml', '.json')):
                profile_path = os.path.join(self.config_dir, filename)
                profile = self.load_profile(profile_path)
                
                if profile:
                    self.profiles[profile.name] = profile
                    
                    # Set the first profile as default if not already set
                    if self.default_profile is None:
                        self.default_profile = profile.name
        
        return self.profiles
    
    def load_profile(self, profile_path: str) -> Optional[ProfileConfig]:
        """Load a profile configuration from a file.
        
        Args:
            profile_path: Path to the profile configuration file
            
        Returns:
            Profile configuration, or None if loading failed
        """
        try:
            with open(profile_path, 'r') as f:
                if profile_path.endswith(('.yaml', '.yml')):
                    data = yaml.safe_load(f)
                else:  # JSON
                    data = json.load(f)
            
            # Create profile configuration
            profile = ProfileConfig(
                name=data.get('name', os.path.basename(profile_path).split('.')[0]),
                description=data.get('description', ''),
                iterations=data.get('iterations', 5),
                warmup_iterations=data.get('warmup_iterations', 2),
                gc_between_runs=data.get('gc_between_runs', True),
                languages=data.get('languages', []),
                output_formats=data.get('output_formats', []),
                output_dir=data.get('output_dir', 'benchmark_reports')
            )
            
            # Load categories
            for category_data in data.get('categories', []):
                category = CategoryConfig(
                    name=category_data.get('name', ''),
                    description=category_data.get('description', ''),
                    enabled=category_data.get('enabled', True)
                )
                
                # Load benchmarks
                for benchmark_data in category_data.get('benchmarks', []):
                    benchmark = BenchmarkConfig(
                        name=benchmark_data.get('name', ''),
                        description=benchmark_data.get('description', ''),
                        enabled=benchmark_data.get('enabled', True),
                        iterations=benchmark_data.get('iterations', profile.iterations),
                        warmup_iterations=benchmark_data.get('warmup_iterations', profile.warmup_iterations),
                        parameters=benchmark_data.get('parameters', {}),
                        tags=benchmark_data.get('tags', [])
                    )
                    
                    category.benchmarks[benchmark.name] = benchmark
                
                profile.categories[category.name] = category
            
            return profile
        
        except Exception as e:
            print(f"Error loading profile from {profile_path}: {e}")
            return None
    
    def save_profile(self, profile: ProfileConfig, profile_path: str = None) -> bool:
        """Save a profile configuration to a file.
        
        Args:
            profile: Profile configuration to save
            profile_path: Path to save the profile to, or None to use the default path
            
        Returns:
            True if saving succeeded, False otherwise
        """
        if profile_path is None:
            profile_path = os.path.join(self.config_dir, f"{profile.name}.yaml")
        
        try:
            # Convert profile to dictionary
            data = {
                'name': profile.name,
                'description': profile.description,
                'iterations': profile.iterations,
                'warmup_iterations': profile.warmup_iterations,
                'gc_between_runs': profile.gc_between_runs,
                'languages': profile.languages,
                'output_formats': profile.output_formats,
                'output_dir': profile.output_dir,
                'categories': []
            }
            
            # Add categories
            for category_name, category in profile.categories.items():
                category_data = {
                    'name': category.name,
                    'description': category.description,
                    'enabled': category.enabled,
                    'benchmarks': []
                }
                
                # Add benchmarks
                for benchmark_name, benchmark in category.benchmarks.items():
                    benchmark_data = {
                        'name': benchmark.name,
                        'description': benchmark.description,
                        'enabled': benchmark.enabled,
                        'iterations': benchmark.iterations,
                        'warmup_iterations': benchmark.warmup_iterations,
                        'parameters': benchmark.parameters,
                        'tags': benchmark.tags
                    }
                    
                    category_data['benchmarks'].append(benchmark_data)
                
                data['categories'].append(category_data)
            
            # Write to file
            with open(profile_path, 'w') as f:
                if profile_path.endswith(('.yaml', '.yml')):
                    yaml.dump(data, f, default_flow_style=False)
                else:  # JSON
                    json.dump(data, f, indent=2)
            
            return True
        
        except Exception as e:
            print(f"Error saving profile to {profile_path}: {e}")
            return False
    
    def create_default_profiles(self) -> None:
        """Create default benchmark profiles if they don't exist."""
        # Quick profile for fast benchmarking
        quick_profile = ProfileConfig(
            name="quick",
            description="Quick benchmarking profile with minimal iterations",
            iterations=3,
            warmup_iterations=1,
            gc_between_runs=True,
            languages=["Anarchy Inference"],
            output_formats=["text", "html"],
            output_dir="benchmark_reports"
        )
        
        # Add core language category
        core_category = CategoryConfig(
            name="core_language",
            description="Core language feature benchmarks"
        )
        
        # Add some benchmarks
        core_category.benchmarks["variables"] = BenchmarkConfig(
            name="variables",
            description="Variable operations benchmark",
            tags=["core", "variables"]
        )
        
        core_category.benchmarks["arithmetic"] = BenchmarkConfig(
            name="arithmetic",
            description="Arithmetic operations benchmark",
            tags=["core", "arithmetic"]
        )
        
        quick_profile.categories["core_language"] = core_category
        
        # Save the quick profile
        self.save_profile(quick_profile)
        
        # Thorough profile for comprehensive benchmarking
        thorough_profile = ProfileConfig(
            name="thorough",
            description="Thorough benchmarking profile with many iterations",
            iterations=10,
            warmup_iterations=3,
            gc_between_runs=True,
            languages=["Anarchy Inference", "Python", "JavaScript"],
            output_formats=["text", "html", "csv"],
            output_dir="benchmark_reports"
        )
        
        # Add all categories
        for category_name in ["core_language", "memory_management", "module_system", "macro_system", "realworld_scenarios"]:
            thorough_profile.categories[category_name] = CategoryConfig(
                name=category_name,
                description=f"{category_name.replace('_', ' ').title()} benchmarks"
            )
        
        # Save the thorough profile
        self.save_profile(thorough_profile)
        
        # Memory-focused profile
        memory_profile = ProfileConfig(
            name="memory",
            description="Memory-focused benchmarking profile",
            iterations=5,
            warmup_iterations=2,
            gc_between_runs=False,  # Don't run GC between runs to measure memory usage
            languages=["Anarchy Inference"],
            output_formats=["text", "html"],
            output_dir="benchmark_reports"
        )
        
        # Add memory management category
        memory_category = CategoryConfig(
            name="memory_management",
            description="Memory management benchmarks"
        )
        
        # Add some benchmarks
        memory_category.benchmarks["object_allocation"] = BenchmarkConfig(
            name="object_allocation",
            description="Object allocation benchmark",
            tags=["memory", "allocation"]
        )
        
        memory_category.benchmarks["gc"] = BenchmarkConfig(
            name="gc",
            description="Garbage collection benchmark",
            tags=["memory", "gc"]
        )
        
        memory_profile.categories["memory_management"] = memory_category
        
        # Save the memory profile
        self.save_profile(memory_profile)
        
        # Cross-language profile
        cross_language_profile = ProfileConfig(
            name="cross_language",
            description="Cross-language comparison profile",
            iterations=5,
            warmup_iterations=2,
            gc_between_runs=True,
            languages=["Anarchy Inference", "Python", "JavaScript"],
            output_formats=["html"],
            output_dir="benchmark_reports"
        )
        
        # Add core language and realworld categories
        for category_name in ["core_language", "realworld_scenarios"]:
            cross_language_profile.categories[category_name] = CategoryConfig(
                name=category_name,
                description=f"{category_name.replace('_', ' ').title()} benchmarks"
            )
        
        # Save the cross-language profile
        self.save_profile(cross_language_profile)
    
    def get_profile(self, profile_name: str = None) -> Optional[ProfileConfig]:
        """Get a profile configuration by name.
        
        Args:
            profile_name: Name of the profile, or None to use the default profile
            
        Returns:
            Profile configuration, or None if not found
        """
        if not self.profiles:
            self.load_profiles()
        
        if profile_name is None:
            profile_name = self.default_profile
        
        return self.profiles.get(profile_name)


# Example usage
if __name__ == "__main__":
    # Create configuration manager
    config_manager = ConfigurationManager()
    
    # Create default profiles
    config_manager.create_default_profiles()
    
    # Load profiles
    profiles = config_manager.load_profiles()
    
    # Print profile names
    print("Available profiles:")
    for profile_name in profiles:
        print(f"- {profile_name}")
    
    # Get the default profile
    default_profile = config_manager.get_profile()
    
    if default_profile:
        print(f"\nDefault profile: {default_profile.name}")
        print(f"Description: {default_profile.description}")
        print(f"Iterations: {default_profile.iterations}")
        print(f"Categories: {', '.join(default_profile.categories.keys())}")
