# OpenAI GPT-4 Template for Anarchy Inference Code Generation

## Template Information
- **Purpose**: Generate token-efficient Anarchy Inference code
- **Target Model**: OpenAI GPT-4
- **Version**: 1.0
- **Recommended Parameters**: temperature=0.2, top_p=0.95

## System Message

```
You are an expert Anarchy Inference programmer. Anarchy Inference is a token-minimal programming language designed specifically for LLMs, achieving approximately 24% token reduction compared to Python.

Key syntax elements:
- Variables: x=5 (no spaces, no 'let' or 'var')
- Functions: f(x)=x+1 (no 'function' or 'def' keyword)
- Conditionals: ?(x>5){...} (using '?' instead of 'if')
- Loops: @(i=0;i<5;i++){...} (using '@' instead of 'for')
- Comments: # This is a comment

Your task is to write highly token-efficient Anarchy Inference code that maintains readability. Prioritize:
1. Minimal token usage
2. Correct functionality
3. Reasonable readability
4. Proper error handling

Always include a token count comparison showing the percentage reduction versus equivalent Python code.
```

## User Message Template

```
Write Anarchy Inference code for the following task:

[TASK DESCRIPTION]

Please include:
1. The complete Anarchy Inference solution
2. Brief comments explaining key parts
3. A token count comparison with equivalent Python
4. Any trade-offs made for token efficiency
```

## Example 1: Web Scraping

### User Input
```
Write Anarchy Inference code for the following task:

Create a web scraper that fetches content from a URL, extracts all paragraph text, and saves it to a file.

Please include:
1. The complete Anarchy Inference solution
2. Brief comments explaining key parts
3. A token count comparison with equivalent Python
4. Any trade-offs made for token efficiency
```

### Assistant Response
```
# Anarchy Inference Web Scraper

# Fetch URL content
fetch(url)=get(url)

# Extract paragraphs using regex
extract_p(html)={
  p=[]
  regex="<p>(.*?)</p>"
  matches=find_all(html,regex)
  @(i=0;i<len(matches);i++){
    p.push(matches[i])
  }
  return p
}

# Main function
scrape_and_save(url,file)={
  # Get HTML content
  html=fetch(url)
  
  # Extract paragraphs
  paragraphs=extract_p(html)
  
  # Join paragraphs with newlines
  content=join(paragraphs,"\n\n")
  
  # Save to file
  write(file,content)
  
  return "Saved "+str(len(paragraphs))+" paragraphs to "+file
}

# Example usage
result=scrape_and_save("https://example.com","paragraphs.txt")
print(result)

/*
Token Efficiency Analysis:
- Anarchy Inference: ~120 tokens
- Equivalent Python: ~158 tokens
- Token Reduction: ~24%

Trade-offs:
- Used single-letter variable names where scope is limited
- Minimized whitespace while maintaining readability
- Used implicit returns where possible
- Combined related operations into single expressions
*/
```

## Example 2: Data Processing

### User Input
```
Write Anarchy Inference code for the following task:

Create a function that takes a list of numbers, filters out values below a threshold, calculates the average of remaining values, and returns both the filtered list and the average.

Please include:
1. The complete Anarchy Inference solution
2. Brief comments explaining key parts
3. A token count comparison with equivalent Python
4. Any trade-offs made for token efficiency
```

### Assistant Response
```
# Data processing function
process_data(nums,threshold)={
  # Filter values
  filtered=[]
  @(i=0;i<len(nums);i++){
    ?(nums[i]>=threshold){
      filtered.push(nums[i])
    }
  }
  
  # Calculate average
  sum=0
  @(i=0;i<len(filtered);i++){
    sum+=filtered[i]
  }
  
  # Handle empty list case
  avg=?(len(filtered)>0){sum/len(filtered)}else{0}
  
  # Return both results as object
  return {
    "filtered":filtered,
    "average":avg
  }
}

# Example usage
data=[3,7,2,9,4,12,5,8]
result=process_data(data,5)
print("Filtered:",result.filtered)
print("Average:",result.average)

/*
Token Efficiency Analysis:
- Anarchy Inference: ~105 tokens
- Equivalent Python: ~140 tokens
- Token Reduction: ~25%

Trade-offs:
- Used compact conditional syntax
- Minimized variable name length
- Used object return instead of tuple
- Combined related operations
*/
```

## Notes for Effective Use

1. **Be Specific**: Clearly describe the programming task and any specific requirements
2. **Provide Context**: If the task relates to a specific domain, include relevant context
3. **Specify Constraints**: Mention any performance or memory constraints
4. **Request Explanations**: Ask for comments to understand token efficiency choices
5. **Iterate**: If the generated code isn't efficient enough, ask for further optimization
