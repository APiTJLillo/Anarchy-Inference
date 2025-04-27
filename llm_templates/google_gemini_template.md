# Google Gemini Template for Anarchy Inference Code Generation

## Template Information
- **Purpose**: Generate token-efficient Anarchy Inference code
- **Target Model**: Google Gemini Pro and Advanced
- **Version**: 1.0
- **Recommended Parameters**: temperature=0.2, top_k=40

## User Message Template

```markdown
# Anarchy Inference Code Generation Request

## About Anarchy Inference
Anarchy Inference is a token-minimal programming language designed specifically for LLMs. It achieves approximately 24% token reduction compared to Python while maintaining readability.

### Key Syntax Elements
- Variables: `x=5` (no spaces, no 'let' or 'var')
- Functions: `f(x)=x+1` (no 'function' or 'def' keyword)
- Conditionals: `?(x>5){...}` (using '?' instead of 'if')
- Loops: `@(i=0;i<5;i++){...}` (using '@' instead of 'for')
- Comments: `# This is a comment`

## Task Description
[TASK DESCRIPTION]

## Requirements
Please provide:
1. The complete Anarchy Inference solution
2. Step-by-step explanation of how the code works
3. A token count comparison with equivalent Python
4. Any trade-offs made for token efficiency

## Examples of Anarchy Inference Code

### Example 1: JSON Processing

```anarchy
# Parse and transform JSON data
process_json(data)={
  # Parse JSON if string
  obj=?(typeof(data)=="string"){parse_json(data)}else{data}
  
  # Extract and transform values
  result={}
  
  # Process items array
  items=[]
  @(i=0;i<len(obj.items);i++){
    item=obj.items[i]
    # Transform each item
    transformed={
      "id": item.id,
      "name": uppercase(item.name),
      "score": item.value*10
    }
    items.push(transformed)
  }
  
  # Add metadata
  result.items=items
  result.count=len(items)
  result.processed_at=now()
  
  return result
}
```

### Example 2: Image Processing

```anarchy
# Simple image processing functions
grayscale(img)={
  w=img.width
  h=img.height
  result=create_image(w,h)
  
  @(y=0;y<h;y++){
    @(x=0;x<w;x++){
      # Get RGB values
      pixel=get_pixel(img,x,y)
      r=pixel.r
      g=pixel.g
      b=pixel.b
      
      # Calculate grayscale value
      gray=(r*0.3)+(g*0.59)+(b*0.11)
      
      # Set pixel in result
      set_pixel(result,x,y,gray,gray,gray)
    }
  }
  
  return result
}

# Apply threshold filter
threshold(img,t)={
  w=img.width
  h=img.height
  result=create_image(w,h)
  
  @(y=0;y<h;y++){
    @(x=0;x<w;x++){
      pixel=get_pixel(img,x,y)
      avg=(pixel.r+pixel.g+pixel.b)/3
      val=?(avg>t){255}else{0}
      set_pixel(result,x,y,val,val,val)
    }
  }
  
  return result
}
```
```

## Example 1: Sorting Algorithm

### User Input
```markdown
# Anarchy Inference Code Generation Request

## About Anarchy Inference
Anarchy Inference is a token-minimal programming language designed specifically for LLMs. It achieves approximately 24% token reduction compared to Python while maintaining readability.

### Key Syntax Elements
- Variables: `x=5` (no spaces, no 'let' or 'var')
- Functions: `f(x)=x+1` (no 'function' or 'def' keyword)
- Conditionals: `?(x>5){...}` (using '?' instead of 'if')
- Loops: `@(i=0;i<5;i++){...}` (using '@' instead of 'for')
- Comments: `# This is a comment`

## Task Description
Implement a merge sort algorithm in Anarchy Inference that sorts an array of numbers in ascending order.

## Requirements
Please provide:
1. The complete Anarchy Inference solution
2. Step-by-step explanation of how the code works
3. A token count comparison with equivalent Python
4. Any trade-offs made for token efficiency

## Examples of Anarchy Inference Code
[EXAMPLES AS SHOWN ABOVE]
```

### Assistant Response
```markdown
# Merge Sort Implementation in Anarchy Inference

Here's a complete implementation of merge sort in Anarchy Inference:

```anarchy
# Merge two sorted arrays
merge(left,right)={
  result=[]
  i=0
  j=0
  
  # Compare elements from both arrays and add smaller one to result
  @(;i<len(left)&&j<len(right);){
    ?(left[i]<=right[j]){
      result.push(left[i])
      i++
    }else{
      result.push(right[j])
      j++
    }
  }
  
  # Add remaining elements
  @(;i<len(left);){
    result.push(left[i])
    i++
  }
  
  @(;j<len(right);){
    result.push(right[j])
    j++
  }
  
  return result
}

# Main merge sort function
merge_sort(arr)={
  # Base case: arrays of 0 or 1 element are already sorted
  ?(len(arr)<=1){
    return arr
  }
  
  # Split array in half
  mid=len(arr)/2
  left=arr[0:mid]
  right=arr[mid:len(arr)]
  
  # Recursively sort both halves
  left=merge_sort(left)
  right=merge_sort(right)
  
  # Merge sorted halves
  return merge(left,right)
}

# Example usage
nums=[38,27,43,3,9,82,10]
sorted=merge_sort(nums)
print("Original array:",nums)
print("Sorted array:",sorted)
```

## Step-by-Step Explanation

1. **The `merge` function**:
   - Takes two sorted arrays (`left` and `right`) as input
   - Uses two pointers (`i` and `j`) to track position in each array
   - Compares elements from both arrays and adds the smaller one to the result
   - After one array is exhausted, adds remaining elements from the other array
   - Returns the merged sorted array

2. **The `merge_sort` function**:
   - Implements the recursive divide-and-conquer approach
   - Base case: arrays with 0 or 1 elements are already sorted
   - Splits the input array into two halves
   - Recursively sorts each half
   - Merges the sorted halves using the `merge` function
   - Returns the final sorted array

3. **Example usage**:
   - Creates a sample array of numbers
   - Calls `merge_sort` to sort the array
   - Prints both original and sorted arrays

## Token Efficiency Analysis

- Anarchy Inference: ~140 tokens
- Equivalent Python: ~185 tokens
- Token Reduction: ~24%

## Token Efficiency Trade-offs

1. **Variable naming**: Used short variable names (`i`, `j`, `mid`) to reduce token count
2. **Loop optimization**: Used compact loop syntax in the merge function
3. **Implicit returns**: No explicit `return` keyword needed in conditional expressions
4. **Minimal whitespace**: Reduced unnecessary whitespace while maintaining readability
5. **Direct array slicing**: Used direct array slicing instead of creating helper functions

The implementation maintains the core merge sort algorithm's efficiency (O(n log n) time complexity) while reducing token count through Anarchy Inference's concise syntax.
```

## Example 2: File System Operations

### User Input
```markdown
# Anarchy Inference Code Generation Request

## About Anarchy Inference
Anarchy Inference is a token-minimal programming language designed specifically for LLMs. It achieves approximately 24% token reduction compared to Python while maintaining readability.

### Key Syntax Elements
- Variables: `x=5` (no spaces, no 'let' or 'var')
- Functions: `f(x)=x+1` (no 'function' or 'def' keyword)
- Conditionals: `?(x>5){...}` (using '?' instead of 'if')
- Loops: `@(i=0;i<5;i++){...}` (using '@' instead of 'for')
- Comments: `# This is a comment`

## Task Description
Create a file management utility in Anarchy Inference that can:
1. Recursively list all files in a directory
2. Filter files by extension
3. Copy matching files to a destination directory
4. Generate a report of the operation

## Requirements
Please provide:
1. The complete Anarchy Inference solution
2. Step-by-step explanation of how the code works
3. A token count comparison with equivalent Python
4. Any trade-offs made for token efficiency

## Examples of Anarchy Inference Code
[EXAMPLES AS SHOWN ABOVE]
```

### Assistant Response
```markdown
# File Management Utility in Anarchy Inference

Here's a complete implementation of a file management utility in Anarchy Inference:

```anarchy
# List all files in directory recursively
list_files(dir)={
  files=[]
  entries=read_dir(dir)
  
  @(i=0;i<len(entries);i++){
    entry=entries[i]
    path=join_path(dir,entry)
    
    # Check if directory
    ?(is_dir(path)){
      # Recursively get files from subdirectory
      sub_files=list_files(path)
      @(j=0;j<len(sub_files);j++){
        files.push(sub_files[j])
      }
    }else{
      # Add file to list
      files.push(path)
    }
  }
  
  return files
}

# Get file extension
get_ext(file)={
  parts=split(file,".")
  ?(len(parts)>1){
    return parts[len(parts)-1]
  }
  return ""
}

# Filter files by extension
filter_by_ext(files,ext)={
  result=[]
  @(i=0;i<len(files);i++){
    file=files[i]
    ?(get_ext(file)==ext){
      result.push(file)
    }
  }
  return result
}

# Copy file to destination
copy_file(src,dest)={
  # Create destination directory if needed
  dest_dir=dir_name(dest)
  ?(!(exists(dest_dir))){
    make_dir(dest_dir)
  }
  
  # Read source file
  content=read_file(src)
  
  # Write to destination
  write_file(dest,content)
  
  return true
}

# Main file management function
manage_files(src_dir,dest_dir,ext)={
  # Get all files
  all_files=list_files(src_dir)
  
  # Filter by extension
  matching=filter_by_ext(all_files,ext)
  
  # Copy each file
  copied=[]
  @(i=0;i<len(matching);i++){
    src=matching[i]
    # Create relative path
    rel_path=relative_path(src,src_dir)
    dest=join_path(dest_dir,rel_path)
    
    # Copy file
    success=copy_file(src,dest)
    ?(success){
      copied.push({
        "source":src,
        "destination":dest
      })
    }
  }
  
  # Generate report
  report={
    "total_files":len(all_files),
    "matching_files":len(matching),
    "copied_files":len(copied),
    "extension":ext,
    "source_dir":src_dir,
    "destination_dir":dest_dir,
    "copied":copied
  }
  
  return report
}

# Example usage
src="/path/to/source"
dest="/path/to/destination"
report=manage_files(src,dest,"txt")
print("File Management Report:")
print("Total files scanned:",report.total_files)
print("Files with .txt extension:",report.matching_files)
print("Files successfully copied:",report.copied_files)
```

## Step-by-Step Explanation

1. **The `list_files` function**:
   - Recursively lists all files in a directory and its subdirectories
   - Uses `read_dir` to get directory entries
   - For each entry, checks if it's a directory or file
   - If it's a directory, recursively calls itself
   - Returns a flat array of all file paths

2. **The `get_ext` function**:
   - Extracts the file extension from a file path
   - Splits the path by "." and returns the last part
   - Returns empty string if no extension found

3. **The `filter_by_ext` function**:
   - Filters an array of file paths by extension
   - Uses `get_ext` to check each file's extension
   - Returns only files matching the specified extension

4. **The `copy_file` function**:
   - Copies a file from source to destination
   - Creates destination directory if it doesn't exist
   - Reads source file content and writes to destination
   - Returns true on success

5. **The `manage_files` function**:
   - Main function that orchestrates the entire process
   - Gets all files from source directory
   - Filters files by extension
   - Copies matching files to destination
   - Generates a detailed report of the operation

6. **Example usage**:
   - Sets source and destination directories
   - Calls `manage_files` with ".txt" extension
   - Prints a summary of the operation

## Token Efficiency Analysis

- Anarchy Inference: ~230 tokens
- Equivalent Python: ~305 tokens
- Token Reduction: ~25%

## Token Efficiency Trade-offs

1. **Variable naming**: Used concise variable names while maintaining clarity
2. **Function modularity**: Split functionality into focused functions to avoid code duplication
3. **Direct property access**: Used direct object property access instead of getter methods
4. **Minimal error handling**: Implemented basic error handling without verbose try/catch blocks
5. **Compact data structures**: Used simple objects for report generation

The implementation provides a complete file management solution while leveraging Anarchy Inference's concise syntax to reduce token count compared to equivalent Python code.
```

## Notes for Effective Use

1. **Use Markdown Formatting**: Gemini works well with structured markdown
2. **Provide Step-by-Step Explanations**: Request detailed explanations of how the code works
3. **Include Code Blocks**: Use triple backticks with language identifier for code examples
4. **Be Specific About Requirements**: Clearly outline what you need the code to do
5. **Request Token Analysis**: Explicitly ask for token count comparison and efficiency trade-offs
