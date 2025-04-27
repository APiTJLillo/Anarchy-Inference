# Open Source LLM Template for Anarchy Inference Code Generation

## Template Information
- **Purpose**: Generate token-efficient Anarchy Inference code
- **Target Models**: Llama 3, Mistral, and other open source LLMs
- **Version**: 1.0
- **Recommended Parameters**: temperature=0.1, top_p=0.9

## Prompt Template

```
You are an expert programmer specializing in Anarchy Inference, a token-minimal programming language designed specifically for LLMs. Your task is to write efficient, correct Anarchy Inference code.

# ANARCHY INFERENCE LANGUAGE SPECIFICATION

Anarchy Inference is a programming language that achieves approximately 24% token reduction compared to Python while maintaining readability.

Key syntax elements:
- Variables: x=5 (no spaces, no 'let' or 'var')
- Functions: f(x)=x+1 (no 'function' or 'def' keyword)
- Conditionals: ?(x>5){...} (using '?' instead of 'if')
- Loops: @(i=0;i<5;i++){...} (using '@' instead of 'for')
- Comments: # This is a comment

# PROGRAMMING TASK

[TASK DESCRIPTION]

# REQUIREMENTS

1. Write complete, working Anarchy Inference code
2. Include comments explaining key parts
3. Focus on token efficiency while maintaining readability
4. Include error handling for edge cases
5. Provide a token count comparison with equivalent Python

# EXAMPLES OF ANARCHY INFERENCE CODE

Example 1: Array manipulation

```anarchy
# Sum all even numbers in an array
sum_even(arr)={
  total=0
  @(i=0;i<len(arr);i++){
    # Check if number is even
    ?(arr[i]%2==0){
      total+=arr[i]
    }
  }
  return total
}

# Filter array by condition
filter(arr,condition)={
  result=[]
  @(i=0;i<len(arr);i++){
    ?(condition(arr[i])){
      result.push(arr[i])
    }
  }
  return result
}

# Example usage
numbers=[1,2,3,4,5,6,7,8,9,10]
is_even(x)=x%2==0
evens=filter(numbers,is_even)
sum=sum_even(numbers)
print("Even numbers:",evens)
print("Sum of even numbers:",sum)
```

Example 2: Error handling

```anarchy
# Safely divide two numbers
safe_divide(a,b)={
  # Check for division by zero
  ?(b==0){
    return {
      "success":false,
      "error":"Division by zero",
      "result":null
    }
  }
  
  # Perform division
  result=a/b
  
  return {
    "success":true,
    "error":null,
    "result":result
  }
}

# Example usage
result1=safe_divide(10,2)
result2=safe_divide(10,0)

?(result1.success){
  print("10/2 =",result1.result)
}else{
  print("Error:",result1.error)
}

?(result2.success){
  print("10/0 =",result2.result)
}else{
  print("Error:",result2.error)
}
```

# YOUR TASK

Now, write Anarchy Inference code for the task described above. Include a token count comparison with equivalent Python code.
```

## Example 1: Binary Search Implementation

### User Input
```
You are an expert programmer specializing in Anarchy Inference, a token-minimal programming language designed specifically for LLMs. Your task is to write efficient, correct Anarchy Inference code.

# ANARCHY INFERENCE LANGUAGE SPECIFICATION

Anarchy Inference is a programming language that achieves approximately 24% token reduction compared to Python while maintaining readability.

Key syntax elements:
- Variables: x=5 (no spaces, no 'let' or 'var')
- Functions: f(x)=x+1 (no 'function' or 'def' keyword)
- Conditionals: ?(x>5){...} (using '?' instead of 'if')
- Loops: @(i=0;i<5;i++){...} (using '@' instead of 'for')
- Comments: # This is a comment

# PROGRAMMING TASK

Implement a binary search algorithm in Anarchy Inference that searches for a target value in a sorted array. The function should return the index of the target if found, or -1 if not found.

# REQUIREMENTS

1. Write complete, working Anarchy Inference code
2. Include comments explaining key parts
3. Focus on token efficiency while maintaining readability
4. Include error handling for edge cases
5. Provide a token count comparison with equivalent Python

# EXAMPLES OF ANARCHY INFERENCE CODE

[EXAMPLES AS SHOWN ABOVE]

# YOUR TASK

Now, write Anarchy Inference code for the task described above. Include a token count comparison with equivalent Python code.
```

### Assistant Response
```
# Binary Search Implementation in Anarchy Inference

# Binary search function
binary_search(arr,target)={
  # Handle edge cases
  ?(len(arr)==0){
    return -1
  }
  
  # Initialize search boundaries
  left=0
  right=len(arr)-1
  
  # Search while boundaries are valid
  @(;left<=right;){
    # Calculate middle index
    mid=(left+right)/2
    
    # Check if target found
    ?(arr[mid]==target){
      return mid
    }
    
    # Adjust boundaries based on comparison
    ?(arr[mid]<target){
      left=mid+1
    }else{
      right=mid-1
    }
  }
  
  # Target not found
  return -1
}

# Test function with examples
test_binary_search()={
  # Test case 1: Target in middle
  arr1=[1,3,5,7,9,11,13]
  result1=binary_search(arr1,7)
  
  # Test case 2: Target at beginning
  arr2=[1,3,5,7,9,11,13]
  result2=binary_search(arr2,1)
  
  # Test case 3: Target at end
  arr3=[1,3,5,7,9,11,13]
  result3=binary_search(arr3,13)
  
  # Test case 4: Target not in array
  arr4=[1,3,5,7,9,11,13]
  result4=binary_search(arr4,6)
  
  # Test case 5: Empty array
  arr5=[]
  result5=binary_search(arr5,5)
  
  # Print results
  print("Test 1 (target in middle):",result1)
  print("Test 2 (target at beginning):",result2)
  print("Test 3 (target at end):",result3)
  print("Test 4 (target not in array):",result4)
  print("Test 5 (empty array):",result5)
}

# Run tests
test_binary_search()

# Token Count Comparison:
# Anarchy Inference: ~120 tokens
# Equivalent Python: ~160 tokens
# Token Reduction: ~25%
#
# Token efficiency achieved through:
# 1. No 'def' or 'function' keywords
# 2. Compact conditional syntax using '?'
# 3. Compact loop syntax using '@'
# 4. Minimal whitespace requirements
# 5. Implicit type handling
```

## Example 2: Simple Calculator Implementation

### User Input
```
You are an expert programmer specializing in Anarchy Inference, a token-minimal programming language designed specifically for LLMs. Your task is to write efficient, correct Anarchy Inference code.

# ANARCHY INFERENCE LANGUAGE SPECIFICATION

Anarchy Inference is a programming language that achieves approximately 24% token reduction compared to Python while maintaining readability.

Key syntax elements:
- Variables: x=5 (no spaces, no 'let' or 'var')
- Functions: f(x)=x+1 (no 'function' or 'def' keyword)
- Conditionals: ?(x>5){...} (using '?' instead of 'if')
- Loops: @(i=0;i<5;i++){...} (using '@' instead of 'for')
- Comments: # This is a comment

# PROGRAMMING TASK

Create a simple calculator in Anarchy Inference that can perform basic arithmetic operations (addition, subtraction, multiplication, division) and handle errors like division by zero.

# REQUIREMENTS

1. Write complete, working Anarchy Inference code
2. Include comments explaining key parts
3. Focus on token efficiency while maintaining readability
4. Include error handling for edge cases
5. Provide a token count comparison with equivalent Python

# EXAMPLES OF ANARCHY INFERENCE CODE

[EXAMPLES AS SHOWN ABOVE]

# YOUR TASK

Now, write Anarchy Inference code for the task described above. Include a token count comparison with equivalent Python code.
```

### Assistant Response
```
# Simple Calculator in Anarchy Inference

# Addition function
add(a,b)={
  return a+b
}

# Subtraction function
subtract(a,b)={
  return a-b
}

# Multiplication function
multiply(a,b)={
  return a*b
}

# Division function with error handling
divide(a,b)={
  # Check for division by zero
  ?(b==0){
    return {
      "success":false,
      "error":"Division by zero",
      "result":null
    }
  }
  
  # Perform division
  return {
    "success":true,
    "error":null,
    "result":a/b
  }
}

# Main calculator function
calculate(a,b,op)={
  # Perform operation based on operator
  ?(op=="+"){
    return {
      "success":true,
      "error":null,
      "result":add(a,b)
    }
  }
  
  ?(op=="-"){
    return {
      "success":true,
      "error":null,
      "result":subtract(a,b)
    }
  }
  
  ?(op=="*"){
    return {
      "success":true,
      "error":null,
      "result":multiply(a,b)
    }
  }
  
  ?(op=="/"){
    return divide(a,b)
  }
  
  # Handle invalid operator
  return {
    "success":false,
    "error":"Invalid operator: "+op,
    "result":null
  }
}

# Test calculator with examples
test_calculator()={
  # Test addition
  result1=calculate(10,5,"+")
  
  # Test subtraction
  result2=calculate(10,5,"-")
  
  # Test multiplication
  result3=calculate(10,5,"*")
  
  # Test division
  result4=calculate(10,5,"/")
  
  # Test division by zero
  result5=calculate(10,0,"/")
  
  # Test invalid operator
  result6=calculate(10,5,"&")
  
  # Print results
  print_result(result1,"10 + 5")
  print_result(result2,"10 - 5")
  print_result(result3,"10 * 5")
  print_result(result4,"10 / 5")
  print_result(result5,"10 / 0")
  print_result(result6,"10 & 5")
}

# Helper function to print results
print_result(result,expression)={
  ?(result.success){
    print(expression+" = "+str(result.result))
  }else{
    print("Error in "+expression+": "+result.error)
  }
}

# Run tests
test_calculator()

# Token Count Comparison:
# Anarchy Inference: ~170 tokens
# Equivalent Python: ~225 tokens
# Token Reduction: ~24%
#
# Token efficiency achieved through:
# 1. Compact function definitions
# 2. Simplified conditional syntax
# 3. No explicit return type declarations
# 4. Minimal whitespace requirements
# 5. Concise error handling pattern
```

## Notes for Effective Use

1. **Use Clear Structure**: Open source models benefit from clearly structured prompts
2. **Provide Detailed Examples**: Include multiple examples that demonstrate the language features
3. **Set Low Temperature**: Use a low temperature (0.1-0.2) for more deterministic outputs
4. **Be Explicit**: Clearly state all requirements and expectations
5. **Include Formatting Guidance**: Specify how the code should be formatted and commented
6. **Request Token Analysis**: Explicitly ask for token count comparison
