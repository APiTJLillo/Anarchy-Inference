# Anarchy Inference Language Reference

## Introduction

Anarchy Inference is a token-minimal programming language designed specifically for LLM-generated code. This reference documentation covers the optimized syntax that achieves significant token efficiency compared to mainstream programming languages.

## Language Fundamentals

### Program Structure

An Anarchy Inference program consists of a module declaration containing one or more functions:

```
m{
  main(){
    // Code goes here
    return 0;
  }
}
```

The `m{}` wrapper defines a module, which serves as a container for functions and variables.

### Variables

Variables are declared implicitly by assignment:

```
x = 42;           // Number
name = "Alice";   // String
items = [];       // Array
user = {};        // Object
```

### Data Types

Anarchy Inference supports the following primitive data types:

- **Numbers**: `42`, `3.14`, `-1`
- **Strings**: `"Hello"`, `'World'`
- **Booleans**: `true`, `false`
- **Arrays**: `[1, 2, 3]`
- **Objects**: `{name: "Alice", age: 30}`
- **Null**: `null`

### Operators

#### Arithmetic Operators
- Addition: `+`
- Subtraction: `-`
- Multiplication: `*`
- Division: `/`
- Modulus: `%`
- Increment: `++`
- Decrement: `--`

#### Comparison Operators
- Equal: `==`
- Not equal: `!=`
- Strict equal: `===`
- Strict not equal: `!==`
- Greater than: `>`
- Less than: `<`
- Greater than or equal: `>=`
- Less than or equal: `<=`

#### Logical Operators
- AND: `&&`
- OR: `||`
- NOT: `!`

#### Assignment Operators
- Assignment: `=`
- Add and assign: `+=`
- Subtract and assign: `-=`
- Multiply and assign: `*=`
- Divide and assign: `/=`

### Control Structures

#### Conditional Statements

```
if(condition){
  // Code to execute if condition is true
}else if(another_condition){
  // Code to execute if another_condition is true
}else{
  // Code to execute if all conditions are false
}
```

#### Loops

**For Loop**:
```
for(i=0; i<10; i++){
  // Code to repeat
}
```

**While Loop**:
```
while(condition){
  // Code to repeat while condition is true
}
```

**For-in Loop** (for objects):
```
for(key in object){
  // Code to execute for each key in object
}
```

**For-of Loop** (for arrays):
```
for(item of array){
  // Code to execute for each item in array
}
```

### Functions

Functions are defined using the following syntax:

```
function_name(param1, param2){
  // Function body
  return result;
}
```

Functions can be called as follows:

```
result = function_name(arg1, arg2);
```

Anonymous functions:

```
callback = function(x){
  return x * 2;
};
```

Arrow functions (shorthand):

```
double = (x) => x * 2;
```

### Error Handling

Anarchy Inference uses try-catch blocks for error handling:

```
try{
  // Code that might throw an error
  result = riskyOperation();
}catch(err){
  // Code to handle errors
  print("Error: " + err);
}
```

## Built-in Functions

### Input/Output

- `print(value)`: Outputs a value to the console
- `input(prompt)`: Reads user input from the console

### String Manipulation

- `length(str)`: Returns the length of a string
- `substring(str, start, end)`: Extracts a portion of a string
- `replace(str, old, new)`: Replaces occurrences of a substring
- `split(str, delimiter)`: Splits a string into an array
- `join(array, delimiter)`: Joins array elements into a string

### Array Operations

- `push(array, item)`: Adds an item to the end of an array
- `pop(array)`: Removes and returns the last item of an array
- `shift(array)`: Removes and returns the first item of an array
- `unshift(array, item)`: Adds an item to the beginning of an array
- `slice(array, start, end)`: Extracts a portion of an array
- `length(array)`: Returns the length of an array

### Math Functions

- `abs(x)`: Returns the absolute value of x
- `round(x)`: Rounds x to the nearest integer
- `floor(x)`: Rounds x down to the nearest integer
- `ceil(x)`: Rounds x up to the nearest integer
- `min(x, y)`: Returns the smaller of x and y
- `max(x, y)`: Returns the larger of x and y
- `random()`: Returns a random number between 0 and 1

### Type Conversion

- `Number(value)`: Converts a value to a number
- `String(value)`: Converts a value to a string
- `Boolean(value)`: Converts a value to a boolean
- `parseInt(str, radix)`: Parses a string and returns an integer
- `parseFloat(str)`: Parses a string and returns a floating-point number

### File Operations

- `read(path)`: Reads the contents of a file
- `write(path, content)`: Writes content to a file
- `append(path, content)`: Appends content to a file
- `exists(path)`: Checks if a file exists
- `readdir(path)`: Lists files in a directory
- `mkdir(path)`: Creates a directory
- `remove(path)`: Deletes a file or directory

### Network Operations

- `get(url, options)`: Performs an HTTP GET request
- `post(url, data, options)`: Performs an HTTP POST request
- `put(url, data, options)`: Performs an HTTP PUT request
- `delete(url, options)`: Performs an HTTP DELETE request

### Regular Expressions

- `regex(pattern, flags)`: Creates a regular expression
- `test(regex, str)`: Tests if a string matches a regex
- `match(regex, str)`: Returns matches of a regex in a string
- `replace(str, regex, replacement)`: Replaces regex matches in a string

## Examples

### Web Scraping Example

```
m{
  main(){
    url="https://example.com";
    print("Fetching "+url);
    
    try{
      r=get(url);
      
      if(r.code!=200){
        print("Error: "+r.code);
        return 0;
      }
      
      c=r.body;
      p=[];
      
      m=regex(c,"<p>(.*?)</p>");
      
      for(i=0;i<m.len;i++){
        p.push(m[i]);
      }
      
      print("Found "+p.len+" paragraphs");
      return 1;
    }catch{
      print("Error occurred");
      return 0;
    }
  }
}
```

### Data Processing Example

```
m{
  main(){
    data=read("data.csv");
    rows=data.split("\n");
    headers=rows[0].split(",");
    
    items=[];
    for(i=1;i<rows.length;i++){
      if(!rows[i].trim()) continue;
      
      values=rows[i].split(",");
      item={};
      
      for(j=0;j<headers.length;j++){
        item[headers[j]]=values[j];
      }
      
      item.age=Number(item.age);
      item.income=Number(item.income);
      items.push(item);
    }
    
    stats={
      count: items.length,
      age: {sum:0, avg:0, min:999, max:0},
      income: {sum:0, avg:0, min:999999, max:0}
    };
    
    for(i=0;i<items.length;i++){
      stats.age.sum+=items[i].age;
      if(items[i].age<stats.age.min) stats.age.min=items[i].age;
      if(items[i].age>stats.age.max) stats.age.max=items[i].age;
      
      stats.income.sum+=items[i].income;
      if(items[i].income<stats.income.min) stats.income.min=items[i].income;
      if(items[i].income>stats.income.max) stats.income.max=items[i].income;
    }
    
    stats.age.avg=stats.age.sum/stats.count;
    stats.income.avg=stats.income.sum/stats.count;
    
    print("Processed "+stats.count+" records");
    print("Age - Min: "+stats.age.min+", Max: "+stats.age.max+", Avg: "+stats.age.avg.toFixed(2));
    print("Income - Min: "+stats.income.min+", Max: "+stats.income.max+", Avg: "+stats.income.avg.toFixed(2));
    
    write("stats.json", JSON.stringify(stats, null, 2));
    print("Statistics saved to stats.json");
    
    return 1;
  }
}
```

## Token Efficiency Guidelines

To maximize token efficiency when writing Anarchy Inference code:

1. **Use ASCII characters** instead of Unicode symbols
2. **Keep variable names short** but meaningful
3. **Minimize comments** in production code
4. **Use implicit typing** rather than explicit declarations
5. **Adopt concise error handling** patterns
6. **Avoid unnecessary whitespace** and line breaks
7. **Reuse variables** when possible instead of creating new ones
8. **Use built-in functions** instead of implementing custom solutions

## Comparison with Other Languages

### Anarchy Inference vs. Python

```
# Python
def calculate_average(numbers):
    total = sum(numbers)
    return total / len(numbers) if numbers else 0
```

```
// Anarchy Inference
calculate_average(numbers){
  total=0;
  for(i=0;i<numbers.length;i++){
    total+=numbers[i];
  }
  return numbers.length ? total/numbers.length : 0;
}
```

### Anarchy Inference vs. JavaScript

```
// JavaScript
function processData(data) {
    const result = data.filter(item => item.value > 0)
                       .map(item => item.value * 2)
                       .reduce((sum, val) => sum + val, 0);
    return result;
}
```

```
// Anarchy Inference
processData(data){
  result=0;
  for(i=0;i<data.length;i++){
    if(data[i].value>0){
      result+=data[i].value*2;
    }
  }
  return result;
}
```

## Best Practices

1. **Prioritize readability** while maintaining token efficiency
2. **Structure code logically** with clear function boundaries
3. **Handle errors appropriately** to ensure robust applications
4. **Document complex logic** with minimal but effective comments
5. **Use consistent naming conventions** for variables and functions
6. **Optimize hot paths** for both token efficiency and performance
7. **Test thoroughly** to ensure correct behavior despite concise syntax

## Conclusion

Anarchy Inference provides a token-efficient alternative to mainstream programming languages, specifically designed for LLM-generated code. By following the syntax and guidelines in this reference, you can create programs that use significantly fewer tokens while maintaining functionality and readability.

For more examples and applications, visit the [Anarchy Inference GitHub repository](https://github.com/APiTJLillo/Anarchy-Inference).
