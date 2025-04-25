#!/bin/bash
# GitHub Pages Deployment Script for Anarchy Inference Website

# Configuration
REPO_URL="https://github.com/APiTJLillo/Anarchy-Inference.git"
WEBSITE_DIR="/home/ubuntu/anarchy_inference/website"
DOMAIN="anarchydevelopment.com"

echo "===== GitHub Pages Deployment for Anarchy Inference ====="
echo "Website directory: $WEBSITE_DIR"
echo "Domain: $DOMAIN"

# Create a temporary directory for deployment
TEMP_DIR=$(mktemp -d)
echo "Created temporary directory: $TEMP_DIR"

# Clone the repository
echo "Cloning repository..."
git clone $REPO_URL $TEMP_DIR/repo
cd $TEMP_DIR/repo

# Create gh-pages branch if it doesn't exist
echo "Setting up gh-pages branch..."
git checkout -b gh-pages

# Remove everything except .git directory
find . -maxdepth 1 ! -name .git -exec rm -rf {} \;

# Copy website files
echo "Copying website files..."
cp -r $WEBSITE_DIR/* .

# Create CNAME file for custom domain
echo "Creating CNAME file for custom domain..."
echo $DOMAIN > CNAME

# Add all files to git
echo "Adding files to git..."
git add .

# Commit changes
echo "Committing changes..."
git commit -m "Deploy website to GitHub Pages"

echo ""
echo "===== Deployment Preparation Complete ====="
echo ""
echo "To complete the deployment, you need to:"
echo "1. Push the gh-pages branch to GitHub:"
echo "   cd $TEMP_DIR/repo"
echo "   git push -u origin gh-pages"
echo ""
echo "2. Configure GitHub Pages in repository settings:"
echo "   - Go to https://github.com/APiTJLillo/Anarchy-Inference/settings/pages"
echo "   - Set Source to 'Deploy from a branch'"
echo "   - Select the 'gh-pages' branch and '/ (root)' folder"
echo "   - Click Save"
echo ""
echo "3. Configure your domain DNS settings:"
echo "   - Add an A record pointing to GitHub Pages IP addresses:"
echo "     185.199.108.153"
echo "     185.199.109.153"
echo "     185.199.110.153"
echo "     185.199.111.153"
echo "   - Add a CNAME record for 'www' pointing to '$DOMAIN'"
echo ""
echo "4. Wait for DNS propagation (typically 24-48 hours)"
echo ""
echo "5. Verify HTTPS is working correctly by visiting:"
echo "   https://$DOMAIN"
echo ""
echo "Temporary directory with prepared deployment: $TEMP_DIR/repo"
