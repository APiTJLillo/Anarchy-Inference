// Token Calculator for Anarchy Inference Demo
// This script creates a simple token counter visualization for the video demonstration

const tokenCounts = {
  python: {
    webScraping: 312,
    dataProcessing: 0,
    apiInteraction: 0
  },
  javascript: {
    webScraping: 0,
    dataProcessing: 285,
    apiInteraction: 0
  },
  rust: {
    webScraping: 0,
    dataProcessing: 0,
    apiInteraction: 498
  },
  anarchy: {
    webScraping: 178,
    dataProcessing: 162,
    apiInteraction: 223
  }
};

// Calculate percentage reductions
const reductions = {
  webScraping: Math.round((1 - tokenCounts.anarchy.webScraping / tokenCounts.python.webScraping) * 100),
  dataProcessing: Math.round((1 - tokenCounts.anarchy.dataProcessing / tokenCounts.javascript.dataProcessing) * 100),
  apiInteraction: Math.round((1 - tokenCounts.anarchy.apiInteraction / tokenCounts.rust.apiInteraction) * 100)
};

// Function to create a visual token counter
function createTokenCounter(containerId, language, example) {
  const container = document.getElementById(containerId);
  const tokenCount = tokenCounts[language][example];
  
  container.innerHTML = `
    <div class="token-counter ${language}">
      <h3>${language.charAt(0).toUpperCase() + language.slice(1)}</h3>
      <div class="token-bar">
        <div class="token-fill" style="width: ${tokenCount / 5}px;"></div>
      </div>
      <div class="token-count">${tokenCount} tokens</div>
    </div>
  `;
  
  if (language === 'anarchy') {
    const reduction = reductions[example];
    container.innerHTML += `
      <div class="reduction-indicator">
        ${reduction}% fewer tokens
      </div>
    `;
  }
}

// Function to create cost calculator
function createCostCalculator(containerId, example) {
  const container = document.getElementById(containerId);
  const pythonTokens = tokenCounts.python.webScraping;
  const jsTokens = tokenCounts.javascript.dataProcessing;
  const rustTokens = tokenCounts.rust.apiInteraction;
  
  const anarchyTokens = tokenCounts.anarchy.webScraping + 
                        tokenCounts.anarchy.dataProcessing + 
                        tokenCounts.anarchy.apiInteraction;
  
  const traditionalTokens = pythonTokens + jsTokens + rustTokens;
  
  const dailyGenerations = 100;
  const daysPerMonth = 30;
  
  const monthlyTraditionalTokens = traditionalTokens * dailyGenerations * daysPerMonth;
  const monthlyAnarchyTokens = anarchyTokens * dailyGenerations * daysPerMonth;
  const tokensSaved = monthlyTraditionalTokens - monthlyAnarchyTokens;
  
  const gpt4Cost = 0.002; // per 1K tokens
  const gpt35Cost = 0.0005; // per 1K tokens
  
  const gpt4Savings = (tokensSaved * gpt4Cost) / 1000;
  const gpt35Savings = (tokensSaved * gpt35Cost) / 1000;
  
  container.innerHTML = `
    <div class="cost-calculator">
      <h3>Monthly Cost Savings</h3>
      <div class="cost-row">
        <div class="cost-label">Traditional Languages:</div>
        <div class="cost-value">${monthlyTraditionalTokens.toLocaleString()} tokens</div>
      </div>
      <div class="cost-row">
        <div class="cost-label">Anarchy Inference:</div>
        <div class="cost-value">${monthlyAnarchyTokens.toLocaleString()} tokens</div>
      </div>
      <div class="cost-row highlight">
        <div class="cost-label">Tokens Saved:</div>
        <div class="cost-value">${tokensSaved.toLocaleString()} tokens</div>
      </div>
      <div class="cost-row">
        <div class="cost-label">GPT-4 Savings:</div>
        <div class="cost-value">$${gpt4Savings.toFixed(2)}/month</div>
      </div>
      <div class="cost-row">
        <div class="cost-label">GPT-3.5 Savings:</div>
        <div class="cost-value">$${gpt35Savings.toFixed(2)}/month</div>
      </div>
      <div class="cost-row highlight">
        <div class="cost-label">Annual GPT-4 Savings:</div>
        <div class="cost-value">$${(gpt4Savings * 12).toFixed(2)}/year</div>
      </div>
    </div>
  `;
}

// CSS for the token counter
const styles = `
  .token-counter {
    font-family: Arial, sans-serif;
    margin-bottom: 20px;
    padding: 10px;
    border-radius: 5px;
    background-color: #f5f5f5;
  }
  
  .python {
    border-left: 5px solid #3572A5;
  }
  
  .javascript {
    border-left: 5px solid #f7df1e;
  }
  
  .rust {
    border-left: 5px solid #dea584;
  }
  
  .anarchy {
    border-left: 5px solid #ff5722;
    background-color: #fff8f5;
  }
  
  .token-bar {
    height: 20px;
    background-color: #e0e0e0;
    border-radius: 10px;
    margin: 10px 0;
    overflow: hidden;
  }
  
  .token-fill {
    height: 100%;
    background-color: #4CAF50;
    transition: width 1s ease-in-out;
  }
  
  .python .token-fill {
    background-color: #3572A5;
  }
  
  .javascript .token-fill {
    background-color: #f7df1e;
  }
  
  .rust .token-fill {
    background-color: #dea584;
  }
  
  .anarchy .token-fill {
    background-color: #ff5722;
  }
  
  .token-count {
    font-weight: bold;
    text-align: right;
  }
  
  .reduction-indicator {
    background-color: #4CAF50;
    color: white;
    padding: 5px 10px;
    border-radius: 5px;
    font-weight: bold;
    display: inline-block;
    margin-top: 10px;
  }
  
  .cost-calculator {
    font-family: Arial, sans-serif;
    background-color: #f9f9f9;
    padding: 15px;
    border-radius: 5px;
    border-left: 5px solid #2196F3;
  }
  
  .cost-row {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 1px solid #e0e0e0;
  }
  
  .cost-label {
    font-weight: bold;
  }
  
  .cost-value {
    color: #2196F3;
  }
  
  .highlight {
    background-color: #e3f2fd;
    padding: 8px;
    margin: 8px 0;
    border-radius: 3px;
  }
  
  .highlight .cost-value {
    color: #ff5722;
    font-weight: bold;
  }
`;

// Add styles to document
document.head.innerHTML += `<style>${styles}</style>`;

// Initialize counters when page loads
window.onload = function() {
  createTokenCounter('python-web-counter', 'python', 'webScraping');
  createTokenCounter('anarchy-web-counter', 'anarchy', 'webScraping');
  
  createTokenCounter('js-data-counter', 'javascript', 'dataProcessing');
  createTokenCounter('anarchy-data-counter', 'anarchy', 'dataProcessing');
  
  createTokenCounter('rust-api-counter', 'rust', 'apiInteraction');
  createTokenCounter('anarchy-api-counter', 'anarchy', 'apiInteraction');
  
  createCostCalculator('cost-calculator', 'all');
};
