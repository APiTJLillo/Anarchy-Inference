# Anthropic Claude Template for Anarchy Inference Code Generation

## Template Information
- **Purpose**: Generate token-efficient Anarchy Inference code
- **Target Model**: Anthropic Claude 3 (Opus, Sonnet, Haiku)
- **Version**: 1.0
- **Recommended Parameters**: temperature=0.3

## Human Message Template

```
I need your help writing code in Anarchy Inference, a token-minimal programming language designed specifically for LLMs. This language achieves approximately 24% token reduction compared to Python while maintaining readability.

<anarchy_inference_syntax>
- Variables: x=5 (no spaces, no 'let' or 'var')
- Functions: f(x)=x+1 (no 'function' or 'def' keyword)
- Conditionals: ?(x>5){...} (using '?' instead of 'if')
- Loops: @(i=0;i<5;i++){...} (using '@' instead of 'for')
- Comments: # This is a comment
</anarchy_inference_syntax>

Here's the task I need to solve:

<task>
[TASK DESCRIPTION]
</task>

Please provide:
1. The complete Anarchy Inference solution
2. Brief comments explaining key parts
3. A token count comparison with equivalent Python
4. Any trade-offs made for token efficiency

Here are two examples of Anarchy Inference code to help you understand the syntax and style:

<example_1>
# File reading and processing
read_and_count(file)={
  # Read file content
  content=read(file)
  
  # Split into lines
  lines=split(content,"\n")
  
  # Count words in each line
  counts=[]
  @(i=0;i<len(lines);i++){
    words=split(lines[i]," ")
    counts.push(len(words))
  }
  
  return counts
}

# Calculate average
avg(nums)={
  s=0
  @(i=0;i<len(nums);i++){
    s+=nums[i]
  }
  return s/len(nums)
}

# Main function
analyze(file)={
  counts=read_and_count(file)
  average=avg(counts)
  return {
    "line_count": len(counts),
    "word_counts": counts,
    "average_words": average
  }
}
</example_1>

<example_2>
# Simple API client
fetch_data(url,params)={
  # Make request
  response=get(url,params)
  
  # Check status
  ?(response.status!=200){
    return {"error":"Failed with status "+str(response.status)}
  }
  
  # Parse JSON
  data=parse_json(response.body)
  
  # Extract relevant fields
  results=[]
  @(i=0;i<len(data.items);i++){
    item=data.items[i]
    results.push({
      "id":item.id,
      "name":item.name,
      "value":item.metrics.value
    })
  }
  
  return results
}

# Filter and sort results
process_results(results,min_value)={
  # Filter by minimum value
  filtered=[]
  @(i=0;i<len(results);i++){
    ?(results[i].value>=min_value){
      filtered.push(results[i])
    }
  }
  
  # Sort by value (descending)
  @(i=0;i<len(filtered)-1;i++){
    @(j=i+1;j<len(filtered);j++){
      ?(filtered[i].value<filtered[j].value){
        temp=filtered[i]
        filtered[i]=filtered[j]
        filtered[j]=temp
      }
    }
  }
  
  return filtered
}
</example_2>
```

## Example 1: Mathematical Operations

### Human Input
```
I need your help writing code in Anarchy Inference, a token-minimal programming language designed specifically for LLMs. This language achieves approximately 24% token reduction compared to Python while maintaining readability.

<anarchy_inference_syntax>
- Variables: x=5 (no spaces, no 'let' or 'var')
- Functions: f(x)=x+1 (no 'function' or 'def' keyword)
- Conditionals: ?(x>5){...} (using '?' instead of 'if')
- Loops: @(i=0;i<5;i++){...} (using '@' instead of 'for')
- Comments: # This is a comment
</anarchy_inference_syntax>

Here's the task I need to solve:

<task>
Create a program that calculates the Fibonacci sequence up to n terms, then finds the sum and average of the sequence.
</task>

Please provide:
1. The complete Anarchy Inference solution
2. Brief comments explaining key parts
3. A token count comparison with equivalent Python
4. Any trade-offs made for token efficiency

Here are two examples of Anarchy Inference code to help you understand the syntax and style:

[EXAMPLES AS SHOWN ABOVE]
```

### Assistant Response
```
I'll create an Anarchy Inference program to calculate the Fibonacci sequence, sum, and average.

# Anarchy Inference Fibonacci Calculator

# Generate Fibonacci sequence up to n terms
fib(n)={
  # Handle edge cases
  ?(n<=0){return []}
  ?(n==1){return [0]}
  ?(n==2){return [0,1]}
  
  # Initialize sequence with first two terms
  seq=[0,1]
  
  # Generate remaining terms
  @(i=2;i<n;i++){
    next=seq[i-1]+seq[i-2]
    seq.push(next)
  }
  
  return seq
}

# Calculate sum of a sequence
sum(seq)={
  total=0
  @(i=0;i<len(seq);i++){
    total+=seq[i]
  }
  return total
}

# Calculate average of a sequence
avg(seq)={
  ?(len(seq)==0){return 0}
  return sum(seq)/len(seq)
}

# Main function to calculate and display results
fibonacci_stats(n)={
  sequence=fib(n)
  sequence_sum=sum(sequence)
  sequence_avg=avg(sequence)
  
  return {
    "sequence": sequence,
    "sum": sequence_sum,
    "average": sequence_avg
  }
}

# Example usage
result=fibonacci_stats(10)
print("Fibonacci Sequence:",result.sequence)
print("Sum:",result.sum)
print("Average:",result.average)

/*
Token Efficiency Analysis:
- Anarchy Inference: ~130 tokens
- Equivalent Python: ~170 tokens
- Token Reduction: ~24%

Trade-offs:
- Used single-letter variable names in loops
- Minimized whitespace while maintaining readability
- Combined related operations where possible
- Used object return format for multiple values
*/
```

## Example 2: Text Processing

### Human Input
```
I need your help writing code in Anarchy Inference, a token-minimal programming language designed specifically for LLMs. This language achieves approximately 24% token reduction compared to Python while maintaining readability.

<anarchy_inference_syntax>
- Variables: x=5 (no spaces, no 'let' or 'var')
- Functions: f(x)=x+1 (no 'function' or 'def' keyword)
- Conditionals: ?(x>5){...} (using '?' instead of 'if')
- Loops: @(i=0;i<5;i++){...} (using '@' instead of 'for')
- Comments: # This is a comment
</anarchy_inference_syntax>

Here's the task I need to solve:

<task>
Create a function that takes a text string, counts the frequency of each word (case-insensitive), and returns the top N most frequent words.
</task>

Please provide:
1. The complete Anarchy Inference solution
2. Brief comments explaining key parts
3. A token count comparison with equivalent Python
4. Any trade-offs made for token efficiency

Here are two examples of Anarchy Inference code to help you understand the syntax and style:

[EXAMPLES AS SHOWN ABOVE]
```

### Assistant Response
```
I'll create an Anarchy Inference program to count word frequencies and return the top N most frequent words.

# Word frequency counter and sorter

# Clean and normalize text
clean_text(text)={
  # Convert to lowercase
  text=lowercase(text)
  
  # Replace punctuation with spaces
  punctuation=".,;:!?\"'()[]{}-_"
  @(i=0;i<len(punctuation);i++){
    text=replace(text,punctuation[i]," ")
  }
  
  return text
}

# Count word frequencies
count_words(text)={
  # Clean the text
  text=clean_text(text)
  
  # Split into words
  words=split(text," ")
  
  # Count frequencies
  counts={}
  @(i=0;i<len(words);i++){
    word=words[i]
    # Skip empty strings
    ?(word==""){continue}
    
    # Increment count
    ?(counts[word]){
      counts[word]+=1
    }else{
      counts[word]=1
    }
  }
  
  return counts
}

# Get top N frequent words
top_words(text,n)={
  # Get word counts
  counts=count_words(text)
  
  # Convert to array of [word, count] pairs
  pairs=[]
  for_each(counts,k,v){
    pairs.push([k,v])
  }
  
  # Sort by count (descending)
  @(i=0;i<len(pairs)-1;i++){
    @(j=i+1;j<len(pairs);j++){
      ?(pairs[i][1]<pairs[j][1]){
        temp=pairs[i]
        pairs[i]=pairs[j]
        pairs[j]=temp
      }
    }
  }
  
  # Take top N
  result=[]
  limit=min(n,len(pairs))
  @(i=0;i<limit;i++){
    result.push({
      "word":pairs[i][0],
      "count":pairs[i][1]
    })
  }
  
  return result
}

# Example usage
sample_text="The quick brown fox jumps over the lazy dog. The fox was quick and the dog was lazy."
result=top_words(sample_text,5)
print("Top 5 words:",result)

/*
Token Efficiency Analysis:
- Anarchy Inference: ~190 tokens
- Equivalent Python: ~250 tokens
- Token Reduction: ~24%

Trade-offs:
- Used compact variable names
- Combined string operations where possible
- Used direct object property access instead of helper functions
- Simplified sorting algorithm
*/
```

## Notes for Effective Use

1. **Use XML-Style Tags**: Claude works well with content enclosed in XML-style tags
2. **Provide Clear Examples**: Include detailed examples that match your use case
3. **Be Explicit About Syntax**: Clearly define the syntax rules in a dedicated section
4. **Request Specific Outputs**: Ask for token comparisons and trade-off explanations
5. **Iterate if Needed**: If the generated code isn't optimal, ask Claude to optimize specific parts
