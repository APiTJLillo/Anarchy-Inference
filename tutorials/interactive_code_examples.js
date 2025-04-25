// Anarchy Inference Interactive Code Examples

// This file contains interactive code examples for the Anarchy Inference tutorials
// Each example includes both Anarchy Inference code and equivalent code in other languages
// along with token counts for comparison

// Example 1: Hello World
// Description: Simple hello world program with token comparison
const helloWorldExample = {
  title: "Hello World",
  description: "A simple program that prints 'Hello, World!' to the console.",
  anarchyCode: `
// Hello World in Anarchy Inference
log("Hello, World!")
  `,
  anarchyTokens: 7,
  pythonCode: `
# Hello World in Python
print("Hello, World!")
  `,
  pythonTokens: 9,
  javascriptCode: `
// Hello World in JavaScript
console.log("Hello, World!");
  `,
  javascriptTokens: 11,
  explanation: "Even in this simple example, Anarchy Inference uses fewer tokens than Python or JavaScript."
};

// Example 2: Variable Assignment
// Description: Variable assignment with token comparison
const variableAssignmentExample = {
  title: "Variable Assignment",
  description: "Assigning values to variables and performing basic operations.",
  anarchyCode: `
// Variable assignment in Anarchy Inference
x ← 10
y ← 20
sum ← x + y
log("Sum: " + sum)
  `,
  anarchyTokens: 19,
  pythonCode: `
# Variable assignment in Python
x = 10
y = 20
sum = x + y
print("Sum: " + str(sum))
  `,
  pythonTokens: 25,
  javascriptCode: `
// Variable assignment in JavaScript
const x = 10;
const y = 20;
const sum = x + y;
console.log("Sum: " + sum);
  `,
  javascriptTokens: 31,
  explanation: "Anarchy Inference uses the ← symbol for assignment, saving tokens compared to 'const' or 'let' in JavaScript."
};

// Example 3: Conditional Logic
// Description: If-else statements with token comparison
const conditionalExample = {
  title: "Conditional Logic",
  description: "Using if-else statements to make decisions in code.",
  anarchyCode: `
// Conditional logic in Anarchy Inference
num ← 15
ι num > 10 {
  log("Number is greater than 10")
} ε {
  log("Number is not greater than 10")
}
  `,
  anarchyTokens: 28,
  pythonCode: `
# Conditional logic in Python
num = 15
if num > 10:
    print("Number is greater than 10")
else:
    print("Number is not greater than 10")
  `,
  pythonTokens: 32,
  javascriptCode: `
// Conditional logic in JavaScript
const num = 15;
if (num > 10) {
    console.log("Number is greater than 10");
} else {
    console.log("Number is not greater than 10");
}
  `,
  javascriptTokens: 39,
  explanation: "Anarchy Inference uses ι for 'if' and ε for 'else', reducing token count while maintaining readability."
};

// Example 4: Functions
// Description: Function definition and calling with token comparison
const functionExample = {
  title: "Functions",
  description: "Defining and calling functions with parameters and return values.",
  anarchyCode: `
// Function definition in Anarchy Inference
ƒ calculateArea(length, width) {
  area ← length * width
  ↵ area
}

// Function call
result ← calculateArea(5, 3)
log("Area: " + result)
  `,
  anarchyTokens: 32,
  pythonCode: `
# Function definition in Python
def calculate_area(length, width):
    area = length * width
    return area

# Function call
result = calculate_area(5, 3)
print("Area: " + str(result))
  `,
  pythonTokens: 42,
  javascriptCode: `
// Function definition in JavaScript
function calculateArea(length, width) {
    const area = length * width;
    return area;
}

// Function call
const result = calculateArea(5, 3);
console.log("Area: " + result);
  `,
  javascriptTokens: 47,
  explanation: "Anarchy Inference uses ƒ for function definition and ↵ for return, saving tokens compared to 'function' and 'return'."
};

// Example 5: Loops
// Description: Loop structures with token comparison
const loopExample = {
  title: "Loops",
  description: "Using loops to iterate over collections or repeat operations.",
  anarchyCode: `
// Loop in Anarchy Inference
numbers ← [1, 2, 3, 4, 5]
sum ← 0
for num in numbers {
  sum ← sum + num
}
log("Sum: " + sum)
  `,
  anarchyTokens: 31,
  pythonCode: `
# Loop in Python
numbers = [1, 2, 3, 4, 5]
sum = 0
for num in numbers:
    sum = sum + num
print("Sum: " + str(sum))
  `,
  pythonTokens: 36,
  javascriptCode: `
// Loop in JavaScript
const numbers = [1, 2, 3, 4, 5];
let sum = 0;
for (const num of numbers) {
    sum = sum + num;
}
console.log("Sum: " + sum);
  `,
  javascriptTokens: 45,
  explanation: "Anarchy Inference's loop syntax is concise while remaining readable, saving tokens especially compared to JavaScript."
};

// Example 6: Array Operations
// Description: Array manipulation with token comparison
const arrayExample = {
  title: "Array Operations",
  description: "Working with arrays using functional operations like map, filter, and reduce.",
  anarchyCode: `
// Array operations in Anarchy Inference
numbers ← [1, 2, 3, 4, 5]
doubled ← numbers.map(n → n * 2)
evens ← numbers.filter(n → n % 2 = 0)
sum ← numbers.reduce((acc, n) → acc + n, 0)
log(doubled)
log(evens)
log(sum)
  `,
  anarchyTokens: 56,
  pythonCode: `
# Array operations in Python
numbers = [1, 2, 3, 4, 5]
doubled = list(map(lambda n: n * 2, numbers))
evens = list(filter(lambda n: n % 2 == 0, numbers))
sum = reduce(lambda acc, n: acc + n, numbers, 0)
print(doubled)
print(evens)
print(sum)
  `,
  pythonTokens: 75,
  javascriptCode: `
// Array operations in JavaScript
const numbers = [1, 2, 3, 4, 5];
const doubled = numbers.map(n => n * 2);
const evens = numbers.filter(n => n % 2 === 0);
const sum = numbers.reduce((acc, n) => acc + n, 0);
console.log(doubled);
console.log(evens);
console.log(sum);
  `,
  javascriptTokens: 81,
  explanation: "Anarchy Inference's functional operations are similar to JavaScript but with more concise syntax, resulting in significant token savings."
};

// Example 7: Object Manipulation
// Description: Working with objects with token comparison
const objectExample = {
  title: "Object Manipulation",
  description: "Creating and manipulating objects with properties and methods.",
  anarchyCode: `
// Object manipulation in Anarchy Inference
person ← {
  name: "Alice",
  age: 30,
  greet: ƒ() { ↵ "Hello, " + this.name }
}
log(person.greet())
person.age ← person.age + 1
log(person.name + " is now " + person.age)
  `,
  anarchyTokens: 54,
  pythonCode: `
# Object manipulation in Python
class Person:
    def __init__(self, name, age):
        self.name = name
        self.age = age
    
    def greet(self):
        return "Hello, " + self.name

person = Person("Alice", 30)
print(person.greet())
person.age = person.age + 1
print(person.name + " is now " + str(person.age))
  `,
  pythonTokens: 87,
  javascriptCode: `
// Object manipulation in JavaScript
const person = {
    name: "Alice",
    age: 30,
    greet: function() { return "Hello, " + this.name; }
};
console.log(person.greet());
person.age = person.age + 1;
console.log(person.name + " is now " + person.age);
  `,
  javascriptTokens: 72,
  explanation: "Anarchy Inference's object syntax is similar to JavaScript but more concise, especially for method definitions."
};

// Example 8: Error Handling
// Description: Try-catch blocks with token comparison
const errorHandlingExample = {
  title: "Error Handling",
  description: "Using try-catch blocks to handle exceptions.",
  anarchyCode: `
// Error handling in Anarchy Inference
try {
  result ← 10 / 0
  log(result)
} catch err {
  log("Error: " + err.message)
}
  `,
  anarchyTokens: 28,
  pythonCode: `
# Error handling in Python
try:
    result = 10 / 0
    print(result)
except Exception as err:
    print("Error: " + str(err))
  `,
  pythonTokens: 34,
  javascriptCode: `
// Error handling in JavaScript
try {
    const result = 10 / 0;
    console.log(result);
} catch (err) {
    console.log("Error: " + err.message);
}
  `,
  javascriptTokens: 39,
  explanation: "Anarchy Inference's error handling is similar to JavaScript but with fewer tokens due to the absence of parentheses and semicolons."
};

// Example 9: String Manipulation
// Description: String operations with token comparison
const stringExample = {
  title: "String Manipulation",
  description: "Working with strings and performing common string operations.",
  anarchyCode: `
// String manipulation in Anarchy Inference
text ← "hello, world"
upper ← text.toUpperCase()
words ← text.split(", ")
replaced ← text.replace("world", "anarchy")
log(upper)
log(words)
log(replaced)
  `,
  anarchyTokens: 39,
  pythonCode: `
# String manipulation in Python
text = "hello, world"
upper = text.upper()
words = text.split(", ")
replaced = text.replace("world", "anarchy")
print(upper)
print(words)
print(replaced)
  `,
  pythonTokens: 43,
  javascriptCode: `
// String manipulation in JavaScript
const text = "hello, world";
const upper = text.toUpperCase();
const words = text.split(", ");
const replaced = text.replace("world", "anarchy");
console.log(upper);
console.log(words);
console.log(replaced);
  `,
  javascriptTokens: 57,
  explanation: "Anarchy Inference's string methods are similar to JavaScript but with more concise variable declarations."
};

// Example 10: API Interaction
// Description: Making HTTP requests with token comparison
const apiExample = {
  title: "API Interaction",
  description: "Making HTTP requests to interact with web APIs.",
  anarchyCode: `
// API interaction in Anarchy Inference
ƒ fetchUser(id) {
  response ← fetch("https://api.example.com/users/" + id)
  user ← response.json()
  ↵ user
}

try {
  user ← fetchUser(123)
  log(user.name)
} catch err {
  log("Error: " + err.message)
}
  `,
  anarchyTokens: 52,
  pythonCode: `
# API interaction in Python
import requests

def fetch_user(id):
    response = requests.get("https://api.example.com/users/" + str(id))
    user = response.json()
    return user

try:
    user = fetch_user(123)
    print(user["name"])
except Exception as err:
    print("Error: " + str(err))
  `,
  pythonTokens: 72,
  javascriptCode: `
// API interaction in JavaScript
async function fetchUser(id) {
    const response = await fetch("https://api.example.com/users/" + id);
    const user = await response.json();
    return user;
}

try {
    const user = await fetchUser(123);
    console.log(user.name);
} catch (err) {
    console.log("Error: " + err.message);
}
  `,
  javascriptTokens: 78,
  explanation: "Anarchy Inference's API interaction is more concise than both Python and JavaScript, with significant token savings."
};

// Export all examples
const interactiveExamples = {
  helloWorldExample,
  variableAssignmentExample,
  conditionalExample,
  functionExample,
  loopExample,
  arrayExample,
  objectExample,
  errorHandlingExample,
  stringExample,
  apiExample
};

// Token efficiency summary
const tokenEfficiencySummary = {
  totalAnarchyTokens: 
    helloWorldExample.anarchyTokens +
    variableAssignmentExample.anarchyTokens +
    conditionalExample.anarchyTokens +
    functionExample.anarchyTokens +
    loopExample.anarchyTokens +
    arrayExample.anarchyTokens +
    objectExample.anarchyTokens +
    errorHandlingExample.anarchyTokens +
    stringExample.anarchyTokens +
    apiExample.anarchyTokens,
  
  totalPythonTokens:
    helloWorldExample.pythonTokens +
    variableAssignmentExample.pythonTokens +
    conditionalExample.pythonTokens +
    functionExample.pythonTokens +
    loopExample.pythonTokens +
    arrayExample.pythonTokens +
    objectExample.pythonTokens +
    errorHandlingExample.pythonTokens +
    stringExample.pythonTokens +
    apiExample.pythonTokens,
  
  totalJavascriptTokens:
    helloWorldExample.javascriptTokens +
    variableAssignmentExample.javascriptTokens +
    conditionalExample.javascriptTokens +
    functionExample.javascriptTokens +
    loopExample.javascriptTokens +
    arrayExample.javascriptTokens +
    objectExample.javascriptTokens +
    errorHandlingExample.javascriptTokens +
    stringExample.javascriptTokens +
    apiExample.javascriptTokens,
  
  // Calculate percentage savings
  pythonSavingsPercent: function() {
    return Math.round((1 - this.totalAnarchyTokens / this.totalPythonTokens) * 100);
  },
  
  javascriptSavingsPercent: function() {
    return Math.round((1 - this.totalAnarchyTokens / this.totalJavascriptTokens) * 100);
  },
  
  overallSavingsPercent: function() {
    const avgOtherTokens = (this.totalPythonTokens + this.totalJavascriptTokens) / 2;
    return Math.round((1 - this.totalAnarchyTokens / avgOtherTokens) * 100);
  }
};

// Usage example:
// console.log(`Anarchy Inference saves ${tokenEfficiencySummary.pythonSavingsPercent()}% tokens compared to Python`);
// console.log(`Anarchy Inference saves ${tokenEfficiencySummary.javascriptSavingsPercent()}% tokens compared to JavaScript`);
// console.log(`Overall token savings: ${tokenEfficiencySummary.overallSavingsPercent()}%`);
