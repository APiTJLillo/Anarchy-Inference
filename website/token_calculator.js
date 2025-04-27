// Update the token calculator JavaScript with the new benchmark data
document.addEventListener('DOMContentLoaded', function() {
    // Get references to all the elements
    const codeSize = document.getElementById('code-size');
    const languageSelect = document.getElementById('language-select');
    const modelSelect = document.getElementById('model-select');
    const customRateContainer = document.getElementById('custom-rate-container');
    const customRate = document.getElementById('custom-rate');
    const monthlyExecutions = document.getElementById('monthly-executions');
    const calculateButton = document.getElementById('calculate-button');
    const resultsContainer = document.getElementById('results-container');
    
    // Result display elements
    const standardTokens = document.getElementById('standard-tokens');
    const anarchyTokens = document.getElementById('anarchy-tokens');
    const tokensSaved = document.getElementById('tokens-saved');
    const standardCost = document.getElementById('standard-cost');
    const anarchyCost = document.getElementById('anarchy-cost');
    const monthlySavings = document.getElementById('monthly-savings');
    const annualSavings = document.getElementById('annual-savings');
    
    // Token efficiency data based on our updated benchmarks
    const tokenEfficiencyData = {
        python: {
            tokensPerLine: 10.4,
            reductionPercentage: 24.3
        },
        javascript: {
            tokensPerLine: 10.2,
            reductionPercentage: 23.4
        },
        rust: {
            tokensPerLine: 12.1,
            reductionPercentage: 35.6
        }
    };
    
    // LLM model pricing data ($ per 1K tokens)
    const modelPricing = {
        gpt4: 0.03,
        gpt35: 0.0015,
        claude: 0.025,
        custom: null // Will be set by user
    };
    
    // Show/hide custom rate input based on model selection
    modelSelect.addEventListener('change', function() {
        if (modelSelect.value === 'custom') {
            customRateContainer.classList.remove('hidden');
        } else {
            customRateContainer.classList.add('hidden');
        }
    });
    
    // Calculate button click handler
    calculateButton.addEventListener('click', function() {
        // Get input values
        const lines = parseInt(codeSize.value) || 1000;
        const language = languageSelect.value;
        const model = modelSelect.value;
        const executions = parseInt(monthlyExecutions.value) || 1000;
        
        // Get token efficiency data for selected language
        const efficiencyData = tokenEfficiencyData[language];
        
        // Calculate token counts
        const standardTokenCount = lines * efficiencyData.tokensPerLine;
        const anarchyTokenCount = Math.round(standardTokenCount * (1 - efficiencyData.reductionPercentage / 100));
        const tokensSavedCount = standardTokenCount - anarchyTokenCount;
        const tokensSavedPercentage = efficiencyData.reductionPercentage;
        
        // Get pricing rate
        let rate = modelPricing[model];
        if (model === 'custom') {
            rate = parseFloat(customRate.value) || 0.02;
        }
        
        // Calculate costs
        const standardTokenCost = (standardTokenCount / 1000) * rate;
        const anarchyTokenCost = (anarchyTokenCount / 1000) * rate;
        const monthlySavingsAmount = (standardTokenCost - anarchyTokenCost) * executions;
        const annualSavingsAmount = monthlySavingsAmount * 12;
        
        // Update the results display
        standardTokens.textContent = standardTokenCount.toLocaleString();
        anarchyTokens.textContent = anarchyTokenCount.toLocaleString();
        tokensSaved.textContent = `${tokensSavedCount.toLocaleString()} (${tokensSavedPercentage.toFixed(1)}%)`;
        
        standardCost.textContent = `$${standardTokenCost.toFixed(4)}`;
        anarchyCost.textContent = `$${anarchyTokenCost.toFixed(4)}`;
        monthlySavings.textContent = `$${monthlySavingsAmount.toFixed(2)}`;
        annualSavings.textContent = `$${annualSavingsAmount.toFixed(2)}`;
        
        // Show results
        resultsContainer.classList.remove('hidden');
        
        // Create and update visualization if needed
        updateVisualization(standardTokenCount, anarchyTokenCount);
    });
    
    // Function to update visualization (can be expanded later)
    function updateVisualization(standardTokens, anarchyTokens) {
        // This is a placeholder for potential visualization code
        // Could add a bar chart or other visualization here
        console.log("Visualization data:", standardTokens, anarchyTokens);
    }
    
    // Initialize with default calculation
    calculateButton.click();
});
