// Anarchy Inference Website JavaScript
document.addEventListener('DOMContentLoaded', function() {
  // Mobile menu toggle
  const mobileMenuButton = document.querySelector('nav button');
  const mobileMenu = document.querySelector('nav .hidden');
  
  if (mobileMenuButton && mobileMenu) {
    mobileMenuButton.addEventListener('click', function() {
      mobileMenu.classList.toggle('hidden');
      mobileMenu.classList.toggle('flex');
      mobileMenu.classList.toggle('flex-col');
      mobileMenu.classList.toggle('absolute');
      mobileMenu.classList.toggle('top-16');
      mobileMenu.classList.toggle('right-0');
      mobileMenu.classList.toggle('bg-white');
      mobileMenu.classList.toggle('shadow-md');
      mobileMenu.classList.toggle('p-4');
      mobileMenu.classList.toggle('rounded-lg');
      mobileMenu.classList.toggle('w-48');
    });
  }
  
  // Smooth scrolling for anchor links
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function(e) {
      e.preventDefault();
      
      const targetId = this.getAttribute('href');
      if (targetId === '#') return;
      
      const targetElement = document.querySelector(targetId);
      if (targetElement) {
        window.scrollTo({
          top: targetElement.offsetTop - 80,
          behavior: 'smooth'
        });
      }
    });
  });
  
  // Animation on scroll
  const animateOnScroll = function() {
    const elements = document.querySelectorAll('.feature-card, .chart-container, pre');
    
    elements.forEach(element => {
      const elementPosition = element.getBoundingClientRect().top;
      const windowHeight = window.innerHeight;
      
      if (elementPosition < windowHeight - 100) {
        element.classList.add('animate-fade-in');
      }
    });
  };
  
  // Run animation check on load and scroll
  animateOnScroll();
  window.addEventListener('scroll', animateOnScroll);
  
  // Dynamic year in footer copyright
  const yearElement = document.querySelector('.copyright-year');
  if (yearElement) {
    yearElement.textContent = new Date().getFullYear();
  }
  
  // Token calculator
  const setupTokenCalculator = function() {
    const calculator = document.getElementById('token-calculator');
    if (!calculator) return;
    
    const inputField = calculator.querySelector('input');
    const languageSelect = calculator.querySelector('select');
    const calculateButton = calculator.querySelector('button');
    const resultElement = calculator.querySelector('.result');
    
    if (!inputField || !languageSelect || !calculateButton || !resultElement) return;
    
    calculateButton.addEventListener('click', function() {
      const lineCount = parseInt(inputField.value) || 0;
      const language = languageSelect.value;
      
      let tokensPerLine = 0;
      let reductionPercentage = 0;
      
      switch (language) {
        case 'python':
          tokensPerLine = 12;
          reductionPercentage = 61.3;
          break;
        case 'javascript':
          tokensPerLine = 13;
          reductionPercentage = 63.8;
          break;
        case 'rust':
          tokensPerLine = 15;
          reductionPercentage = 72.1;
          break;
        default:
          tokensPerLine = 12;
          reductionPercentage = 60;
      }
      
      const standardTokens = lineCount * tokensPerLine;
      const anarchyTokens = Math.round(standardTokens * (1 - reductionPercentage / 100));
      const tokensSaved = standardTokens - anarchyTokens;
      
      resultElement.innerHTML = `
        <div class="mt-4 p-4 bg-indigo-50 rounded-lg">
          <p><strong>${language.charAt(0).toUpperCase() + language.slice(1)} tokens:</strong> ${standardTokens.toLocaleString()}</p>
          <p><strong>Anarchy Inference tokens:</strong> ${anarchyTokens.toLocaleString()}</p>
          <p><strong>Tokens saved:</strong> ${tokensSaved.toLocaleString()} (${reductionPercentage}%)</p>
        </div>
      `;
      
      resultElement.classList.remove('hidden');
    });
  };
  
  setupTokenCalculator();
});
