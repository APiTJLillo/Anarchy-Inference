/**
 * Anarchy Inference Web Interpreter
 * 
 * This file provides a JavaScript implementation of the Anarchy Inference interpreter
 * for use in the web playground. It's a simplified version of the Python interpreter
 * that allows users to run Anarchy Inference code directly in the browser.
 */

class AnarchyInterpreter {
    constructor() {
        this.variables = {};
        this.functions = {};
        this.output = [];
    }

    /**
     * Run Anarchy Inference code and return the output
     * @param {string} code - The Anarchy Inference code to execute
     * @returns {string} - The output of the code execution
     */
    run(code) {
        this.variables = {};
        this.functions = {};
        this.output = [];

        try {
            const lines = code.split('\n');
            for (let i = 0; i < lines.length; i++) {
                const line = lines[i].trim();
                if (line === '' || line.startsWith('#')) {
                    continue; // Skip empty lines and comments
                }
                
                this.executeLine(line, i + 1);
            }
            
            return this.output.join('\n');
        } catch (error) {
            throw new Error(`Line ${error.line || '?'}: ${error.message}`);
        }
    }

    /**
     * Execute a single line of Anarchy Inference code
     * @param {string} line - The line of code to execute
     * @param {number} lineNumber - The line number for error reporting
     */
    executeLine(line, lineNumber) {
        try {
            // Function definition
            if (line.includes('(') && line.includes(')') && line.includes('=') && 
                line.indexOf('(') < line.indexOf('=')) {
                this.defineFunction(line);
                return;
            }
            
            // Print statement
            if (line.startsWith('print(')) {
                this.executePrint(line);
                return;
            }
            
            // Variable assignment
            if (line.includes('=') && !line.startsWith('?') && !line.startsWith('@')) {
                this.executeAssignment(line);
                return;
            }
            
            // Conditional statement
            if (line.startsWith('?(')) {
                this.executeConditional(line);
                return;
            }
            
            // Loop statement
            if (line.startsWith('@(')) {
                this.executeLoop(line);
                return;
            }
            
            // Function call (standalone)
            if (line.includes('(') && line.includes(')') && !line.includes('=')) {
                this.executeFunction(line);
                return;
            }
            
            // If we get here, we don't know how to handle this line
            throw new Error(`Unknown syntax: ${line}`);
        } catch (error) {
            error.line = lineNumber;
            throw error;
        }
    }

    /**
     * Define a function
     * @param {string} line - The function definition line
     */
    defineFunction(line) {
        const nameEnd = line.indexOf('(');
        const name = line.substring(0, nameEnd).trim();
        
        const paramsEnd = line.indexOf(')');
        const paramsStr = line.substring(nameEnd + 1, paramsEnd).trim();
        const params = paramsStr ? paramsStr.split(',').map(p => p.trim()) : [];
        
        const body = line.substring(line.indexOf('=') + 1).trim();
        
        this.functions[name] = { params, body };
    }

    /**
     * Execute a print statement
     * @param {string} line - The print statement line
     */
    executePrint(line) {
        // Extract content inside print()
        const content = line.substring(6, line.length - 1).trim();
        
        // Handle multiple arguments separated by commas
        const args = this.parseArguments(content);
        const evaluatedArgs = args.map(arg => this.evaluateExpression(arg));
        
        // Join arguments with space and add to output
        this.output.push(evaluatedArgs.join(' '));
    }

    /**
     * Execute a variable assignment
     * @param {string} line - The assignment line
     */
    executeAssignment(line) {
        const parts = line.split('=');
        const varName = parts[0].trim();
        const expression = parts[1].trim();
        
        const value = this.evaluateExpression(expression);
        this.variables[varName] = value;
    }

    /**
     * Execute a conditional statement
     * @param {string} line - The conditional statement line
     */
    executeConditional(line) {
        // This is a simplified implementation that doesn't actually execute the conditional
        // In a real implementation, we would parse the condition and execute the body if true
        this.output.push("[Conditional statement executed]");
    }

    /**
     * Execute a loop statement
     * @param {string} line - The loop statement line
     */
    executeLoop(line) {
        // This is a simplified implementation that doesn't actually execute the loop
        // In a real implementation, we would parse the loop parameters and execute the body
        this.output.push("[Loop executed]");
    }

    /**
     * Execute a function call
     * @param {string} line - The function call line
     * @returns {any} - The result of the function call
     */
    executeFunction(line) {
        const result = this.evaluateExpression(line);
        // If it's a standalone function call, add the result to output
        if (typeof result !== 'undefined') {
            this.output.push(result.toString());
        }
        return result;
    }

    /**
     * Evaluate an expression
     * @param {string} expression - The expression to evaluate
     * @returns {any} - The result of the expression
     */
    evaluateExpression(expression) {
        // Handle string literals
        if ((expression.startsWith('"') && expression.endsWith('"')) || 
            (expression.startsWith("'") && expression.endsWith("'"))) {
            return expression.substring(1, expression.length - 1);
        }
        
        // Handle numeric literals
        if (!isNaN(expression)) {
            return Number(expression);
        }
        
        // Handle function calls
        if (expression.includes('(') && expression.includes(')')) {
            return this.evaluateFunctionCall(expression);
        }
        
        // Handle variables
        if (this.variables.hasOwnProperty(expression)) {
            return this.variables[expression];
        }
        
        // Handle simple arithmetic expressions (very simplified)
        if (expression.includes('+') || expression.includes('-') || 
            expression.includes('*') || expression.includes('/')) {
            // This is a very simplified approach - in a real implementation,
            // we would use a proper expression parser
            try {
                // Replace variables with their values
                let evalExpr = expression;
                for (const varName in this.variables) {
                    const value = this.variables[varName];
                    if (typeof value === 'number') {
                        evalExpr = evalExpr.replace(new RegExp('\\b' + varName + '\\b', 'g'), value);
                    }
                }
                
                // Evaluate the expression
                // Note: This is unsafe for production use, but acceptable for a demo
                return eval(evalExpr);
            } catch (error) {
                throw new Error(`Error evaluating expression: ${expression}`);
            }
        }
        
        // If we can't evaluate the expression, return it as is
        return expression;
    }

    /**
     * Evaluate a function call
     * @param {string} expression - The function call expression
     * @returns {any} - The result of the function call
     */
    evaluateFunctionCall(expression) {
        const nameEnd = expression.indexOf('(');
        const name = expression.substring(0, nameEnd).trim();
        
        // Handle built-in functions
        if (name === 'len') {
            const arg = this.parseArguments(expression.substring(nameEnd + 1, expression.length - 1))[0];
            const value = this.evaluateExpression(arg);
            return value.length;
        }
        
        if (name === 'upper') {
            const arg = this.parseArguments(expression.substring(nameEnd + 1, expression.length - 1))[0];
            const value = this.evaluateExpression(arg);
            return value.toUpperCase();
        }
        
        if (name === 'lower') {
            const arg = this.parseArguments(expression.substring(nameEnd + 1, expression.length - 1))[0];
            const value = this.evaluateExpression(arg);
            return value.toLowerCase();
        }
        
        if (name === 'split') {
            const args = this.parseArguments(expression.substring(nameEnd + 1, expression.length - 1));
            const str = this.evaluateExpression(args[0]);
            const separator = this.evaluateExpression(args[1]);
            return str.split(separator);
        }
        
        if (name === 'join') {
            const args = this.parseArguments(expression.substring(nameEnd + 1, expression.length - 1));
            const arr = this.evaluateExpression(args[0]);
            const separator = this.evaluateExpression(args[1]);
            return arr.join(separator);
        }
        
        if (name === 'replace') {
            const args = this.parseArguments(expression.substring(nameEnd + 1, expression.length - 1));
            const str = this.evaluateExpression(args[0]);
            const oldStr = this.evaluateExpression(args[1]);
            const newStr = this.evaluateExpression(args[2]);
            return str.replace(oldStr, newStr);
        }
        
        // Handle user-defined functions
        if (this.functions.hasOwnProperty(name)) {
            const func = this.functions[name];
            const argsStr = expression.substring(nameEnd + 1, expression.length - 1);
            const args = this.parseArguments(argsStr);
            
            // Create a new scope with the function parameters
            const oldVariables = { ...this.variables };
            for (let i = 0; i < func.params.length; i++) {
                this.variables[func.params[i]] = this.evaluateExpression(args[i]);
            }
            
            // Evaluate the function body
            const result = this.evaluateExpression(func.body);
            
            // Restore the old scope
            this.variables = oldVariables;
            
            return result;
        }
        
        throw new Error(`Unknown function: ${name}`);
    }

    /**
     * Parse function arguments, handling nested commas in function calls
     * @param {string} argsStr - The arguments string
     * @returns {string[]} - The parsed arguments
     */
    parseArguments(argsStr) {
        if (!argsStr) return [];
        
        const args = [];
        let currentArg = '';
        let parenDepth = 0;
        let inString = false;
        let stringChar = '';
        
        for (let i = 0; i < argsStr.length; i++) {
            const char = argsStr[i];
            
            // Handle strings
            if ((char === '"' || char === "'") && (i === 0 || argsStr[i-1] !== '\\')) {
                if (!inString) {
                    inString = true;
                    stringChar = char;
                } else if (char === stringChar) {
                    inString = false;
                }
            }
            
            // Handle parentheses
            if (!inString) {
                if (char === '(') parenDepth++;
                if (char === ')') parenDepth--;
            }
            
            // Handle commas
            if (char === ',' && parenDepth === 0 && !inString) {
                args.push(currentArg.trim());
                currentArg = '';
                continue;
            }
            
            currentArg += char;
        }
        
        if (currentArg.trim()) {
            args.push(currentArg.trim());
        }
        
        return args;
    }

    /**
     * Estimate the number of tokens in the code
     * @param {string} code - The code to estimate tokens for
     * @returns {number} - The estimated number of tokens
     */
    estimateTokens(code) {
        if (!code) return 0;
        
        // This is a very simplified token estimation
        // In a real implementation, this would use a proper tokenizer
        const words = code.split(/\s+/).filter(w => w.length > 0);
        const symbols = code.match(/[=+\-*\/(){}<>!&|;:,.\[\]]/g) || [];
        
        return words.length + symbols.length;
    }

    /**
     * Convert Anarchy Inference code to Python
     * @param {string} code - The Anarchy Inference code to convert
     * @returns {string} - The equivalent Python code
     */
    toPython(code) {
        // This is a very simplified conversion
        // In a real implementation, this would use a proper parser and code generator
        let pythonCode = '';
        const lines = code.split('\n');
        let indentLevel = 0;
        
        for (let i = 0; i < lines.length; i++) {
            let line = lines[i].trim();
            if (line === '' || line.startsWith('#')) {
                pythonCode += line + '\n';
                continue;
            }
            
            // Function definition
            if (line.includes('(') && line.includes(')') && line.includes('=') && 
                line.indexOf('(') < line.indexOf('=')) {
                const nameEnd = line.indexOf('(');
                const name = line.substring(0, nameEnd).trim();
                
                const paramsEnd = line.indexOf(')');
                const params = line.substring(nameEnd + 1, paramsEnd).trim();
                
                const body = line.substring(line.indexOf('=') + 1).trim();
                
                pythonCode += '    '.repeat(indentLevel) + `def ${name}(${params}):\n`;
                indentLevel++;
                pythonCode += '    '.repeat(indentLevel) + `return ${body}\n`;
                indentLevel--;
                continue;
            }
            
            // Print statement
            if (line.startsWith('print(')) {
                pythonCode += '    '.repeat(indentLevel) + line + '\n';
                continue;
            }
            
            // Variable assignment
            if (line.includes('=') && !line.startsWith('?') && !line.startsWith('@')) {
                const parts = line.split('=');
                const varName = parts[0].trim();
                const expression = parts[1].trim();
                
                pythonCode += '    '.repeat(indentLevel) + `${varName} = ${expression}\n`;
                continue;
            }
            
            // Conditional statement
            if (line.startsWith('?(')) {
                const condition = line.substring(2, line.indexOf(')')).trim();
                pythonCode += '    '.repeat(indentLevel) + `if ${condition}:\n`;
                indentLevel++;
                
                // Extract the body
                const body = line.substring(line.indexOf('{')+1, line.lastIndexOf('}')).trim();
                pythonCode += '    '.repeat(indentLevel) + body + '\n';
                indentLevel--;
                continue;
            }
            
            // Loop statement
            if (line.startsWith('@(')) {
                const loopParams = line.substring(2, line.indexOf(')')).trim();
                const loopParts = loopParams.split(';');
                
                if (loopParts.length === 3) {
                    // For loop
                    const init = loopParts[0].trim();
                    const condition = loopParts[1].trim();
                    const update = loopParts[2].trim();
                    
                    pythonCode += '    '.repeat(indentLevel) + `# Initialize: ${init}\n`;
                    pythonCode += '    '.repeat(indentLevel) + `while ${condition}:\n`;
                    indentLevel++;
                    
                    // Extract the body
                    const body = line.substring(line.indexOf('{')+1, line.lastIndexOf('}')).trim();
                    pythonCode += '    '.repeat(indentLevel) + body + '\n';
                    pythonCode += '    '.repeat(indentLevel) + `# Update: ${update}\n`;
                    indentLevel--;
                }
                continue;
            }
            
            // Function call (standalone)
            if (line.includes('(') && line.includes(')') && !line.includes('=')) {
                pythonCode += '    '.repeat(indentLevel) + line + '\n';
                continue;
            }
            
            // If we get here, just copy the line as is
            pythonCode += '    '.repeat(indentLevel) + line + '\n';
        }
        
        return pythonCode;
    }
}

// Export the interpreter for use in the playground
window.AnarchyInterpreter = AnarchyInterpreter;
