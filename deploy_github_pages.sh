#!/bin/bash

# GitHub Pages Deployment Script for Anarchy Inference
# This script prepares and deploys the website to GitHub Pages

echo "Starting GitHub Pages deployment for Anarchy Inference..."

# Configuration
REPO_URL="https://github.com/APiTJLillo/Anarchy-Inference.git"
DOMAIN="anarchydevelopment.com"
WEBSITE_DIR="/home/ubuntu/anarchy_inference/website"
LANGUAGE_REF="/home/ubuntu/anarchy_inference/language_reference.md"
BENCHMARK_RESULTS="/home/ubuntu/anarchy_inference/benchmark_results/benchmark_report_optimized.md"
TEMP_DIR="/tmp/anarchy_deploy"

# Create temporary directory
echo "Creating temporary deployment directory..."
mkdir -p $TEMP_DIR
cd $TEMP_DIR

# Clone repository (if this is being run locally)
echo "Cloning repository..."
git clone $REPO_URL .
if [ $? -ne 0 ]; then
    echo "Error: Failed to clone repository. Make sure the URL is correct and you have access."
    exit 1
fi

# Create and switch to gh-pages branch
echo "Setting up gh-pages branch..."
git checkout -b gh-pages
if [ $? -ne 0 ]; then
    # Branch might already exist
    git checkout gh-pages
    if [ $? -ne 0 ]; then
        echo "Error: Failed to create or switch to gh-pages branch."
        exit 1
    fi
fi

# Remove all existing files except .git directory
echo "Cleaning branch for fresh deployment..."
find . -mindepth 1 -maxdepth 1 -not -path "./.git" -exec rm -rf {} \;

# Copy website files
echo "Copying website files..."
cp -r $WEBSITE_DIR/* .

# Copy language reference and benchmark results
echo "Copying documentation files..."
cp $LANGUAGE_REF .
cp $BENCHMARK_RESULTS .

# Create CNAME file for custom domain
echo "Setting up custom domain..."
echo $DOMAIN > CNAME

# Add all files to git
echo "Adding files to git..."
git add .

# Commit changes
echo "Committing changes..."
git commit -m "Deploy website to GitHub Pages"
if [ $? -ne 0 ]; then
    echo "Error: Failed to commit changes. Make sure git is configured properly."
    exit 1
fi

echo "==============================================================="
echo "Deployment preparation complete!"
echo "==============================================================="
echo ""
echo "To complete the deployment, follow these steps:"
echo ""
echo "1. Push the gh-pages branch to GitHub:"
echo "   cd $TEMP_DIR"
echo "   git push -u origin gh-pages"
echo ""
echo "2. Go to your repository settings on GitHub:"
echo "   https://github.com/APiTJLillo/Anarchy-Inference/settings/pages"
echo ""
echo "3. Under 'Source', select the 'gh-pages' branch and click 'Save'"
echo ""
echo "4. Configure your domain's DNS settings with the following records:"
echo "   - A record: @ pointing to 185.199.108.153"
echo "   - A record: @ pointing to 185.199.109.153"
echo "   - A record: @ pointing to 185.199.110.153"
echo "   - A record: @ pointing to 185.199.111.153"
echo "   - CNAME record: www pointing to $DOMAIN"
echo ""
echo "5. Wait for DNS propagation (may take up to 24 hours)"
echo ""
echo "Once completed, your website will be available at:"
echo "https://$DOMAIN"
echo ""
echo "==============================================================="
