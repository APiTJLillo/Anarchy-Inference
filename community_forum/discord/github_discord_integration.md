# GitHub-Discord Integration Implementation

## Overview
This document provides the technical implementation details for integrating GitHub with the Anarchy Inference Discord server. This integration will ensure seamless communication between code development activities and community discussions.

## Integration Components

### 1. GitHub Bot for Discord

#### Setup Instructions

1. **Add GitHub Bot to Discord Server**
   ```
   https://discord.com/oauth2/authorize?client_id=84607697821209600&scope=bot&permissions=67584
   ```

2. **Configure Repository Subscription**
   In the `#github-activity` channel, run:
   ```
   /github subscribe APiTJLillo/Anarchy-Inference
   ```

3. **Customize Notification Settings**
   ```
   /github configure APiTJLillo/Anarchy-Inference
   ```
   
   Recommended settings:
   - Issues: ✅
   - Pull Requests: ✅
   - Discussions: ✅
   - Commits: ✅ (main branch only)
   - Releases: ✅
   - Wiki: ✅
   - Stars: ❌ (to reduce noise)
   - Forks: ❌ (to reduce noise)

#### Channel Organization

Create dedicated channels for different GitHub activities:
- `#github-issues` - For issue-related notifications
- `#github-prs` - For pull request notifications
- `#github-commits` - For commit notifications
- `#github-releases` - For release notifications

### 2. Custom Discord Bot with GitHub API Integration

#### Bot Architecture

```javascript
// bot.js - Main bot file
const { Client, Intents, Collection } = require('discord.js');
const { token, githubToken } = require('./config.json');
const fs = require('fs');
const path = require('path');
const { Octokit } = require('@octokit/rest');

// Create Discord client
const client = new Client({
  intents: [
    Intents.FLAGS.GUILDS,
    Intents.FLAGS.GUILD_MESSAGES,
    Intents.FLAGS.GUILD_MEMBERS
  ]
});

// Initialize GitHub API client
const octokit = new Octokit({ auth: githubToken });

// Command collection
client.commands = new Collection();
const commandsPath = path.join(__dirname, 'commands');
const commandFiles = fs.readdirSync(commandsPath).filter(file => file.endsWith('.js'));

for (const file of commandFiles) {
  const filePath = path.join(commandsPath, file);
  const command = require(filePath);
  client.commands.set(command.data.name, command);
}

// Event handling
const eventsPath = path.join(__dirname, 'events');
const eventFiles = fs.readdirSync(eventsPath).filter(file => file.endsWith('.js'));

for (const file of eventFiles) {
  const filePath = path.join(eventsPath, file);
  const event = require(filePath);
  if (event.once) {
    client.once(event.name, (...args) => event.execute(...args, client));
  } else {
    client.on(event.name, (...args) => event.execute(...args, client));
  }
}

// Make GitHub API available to commands
client.github = {
  octokit,
  owner: 'APiTJLillo',
  repo: 'Anarchy-Inference'
};

// Login to Discord
client.login(token);
```

#### Command Implementation

```javascript
// commands/issue.js - Example command for issue management
const { SlashCommandBuilder } = require('@discordjs/builders');
const { MessageEmbed } = require('discord.js');

module.exports = {
  data: new SlashCommandBuilder()
    .setName('issue')
    .setDescription('Manage GitHub issues')
    .addSubcommand(subcommand =>
      subcommand
        .setName('list')
        .setDescription('List open issues')
        .addStringOption(option =>
          option.setName('label')
            .setDescription('Filter by label')
            .setRequired(false)))
    .addSubcommand(subcommand =>
      subcommand
        .setName('view')
        .setDescription('View a specific issue')
        .addIntegerOption(option =>
          option.setName('number')
            .setDescription('Issue number')
            .setRequired(true)))
    .addSubcommand(subcommand =>
      subcommand
        .setName('create')
        .setDescription('Create a new issue')
        .addStringOption(option =>
          option.setName('title')
            .setDescription('Issue title')
            .setRequired(true))
        .addStringOption(option =>
          option.setName('body')
            .setDescription('Issue description')
            .setRequired(true))
        .addStringOption(option =>
          option.setName('labels')
            .setDescription('Comma-separated labels')
            .setRequired(false))),
            
  async execute(interaction) {
    const { github } = interaction.client;
    const subcommand = interaction.options.getSubcommand();
    
    await interaction.deferReply();
    
    try {
      if (subcommand === 'list') {
        const label = interaction.options.getString('label');
        const params = { 
          owner: github.owner, 
          repo: github.repo,
          state: 'open',
          per_page: 10
        };
        
        if (label) params.labels = label;
        
        const { data: issues } = await github.octokit.issues.listForRepo(params);
        
        if (issues.length === 0) {
          return interaction.editReply('No open issues found.');
        }
        
        const embed = new MessageEmbed()
          .setTitle(`Open Issues${label ? ` with label: ${label}` : ''}`)
          .setColor('#0366d6')
          .setURL(`https://github.com/${github.owner}/${github.repo}/issues`);
          
        issues.forEach(issue => {
          embed.addField(
            `#${issue.number}: ${issue.title}`,
            `Opened by ${issue.user.login} | [View Issue](${issue.html_url})`
          );
        });
        
        return interaction.editReply({ embeds: [embed] });
      }
      
      else if (subcommand === 'view') {
        const issueNumber = interaction.options.getInteger('number');
        
        try {
          const { data: issue } = await github.octokit.issues.get({
            owner: github.owner,
            repo: github.repo,
            issue_number: issueNumber
          });
          
          const embed = new MessageEmbed()
            .setTitle(`Issue #${issue.number}: ${issue.title}`)
            .setColor('#0366d6')
            .setURL(issue.html_url)
            .setDescription(issue.body.length > 1024 ? issue.body.substring(0, 1021) + '...' : issue.body)
            .addField('Status', issue.state, true)
            .addField('Created By', issue.user.login, true)
            .setFooter({ text: `Created at ${new Date(issue.created_at).toLocaleString()}` });
            
          if (issue.labels.length > 0) {
            embed.addField('Labels', issue.labels.map(label => label.name).join(', '), true);
          }
          
          return interaction.editReply({ embeds: [embed] });
        } catch (error) {
          if (error.status === 404) {
            return interaction.editReply(`Issue #${issueNumber} not found.`);
          }
          throw error;
        }
      }
      
      else if (subcommand === 'create') {
        // Check if user has permission to create issues
        const member = interaction.member;
        if (!member.roles.cache.some(role => ['Admin', 'Core Contributor', 'Moderator'].includes(role.name))) {
          return interaction.editReply('You do not have permission to create issues.');
        }
        
        const title = interaction.options.getString('title');
        const body = interaction.options.getString('body');
        const labelsString = interaction.options.getString('labels');
        const labels = labelsString ? labelsString.split(',').map(label => label.trim()) : [];
        
        const { data: issue } = await github.octokit.issues.create({
          owner: github.owner,
          repo: github.repo,
          title,
          body: `${body}\n\n_Created via Discord by ${interaction.user.tag}_`,
          labels
        });
        
        const embed = new MessageEmbed()
          .setTitle('Issue Created')
          .setColor('#2cbe4e')
          .setDescription(`Successfully created issue #${issue.number}: ${issue.title}`)
          .addField('Link', issue.html_url);
          
        return interaction.editReply({ embeds: [embed] });
      }
    } catch (error) {
      console.error(error);
      return interaction.editReply('An error occurred while processing your request.');
    }
  },
};
```

#### Additional Commands

Implement the following commands:

1. **PR Management**
   ```javascript
   // commands/pr.js
   // - List pull requests
   // - View specific PR
   // - Create PR from branch
   ```

2. **Repository Information**
   ```javascript
   // commands/repo.js
   // - View repository stats
   // - List contributors
   // - Show recent activity
   ```

3. **Code Snippets**
   ```javascript
   // commands/code.js
   // - View file content
   // - Search code
   // - Share snippets
   ```

4. **Workflow Management**
   ```javascript
   // commands/workflow.js
   // - List workflows
   // - View workflow runs
   // - Trigger workflow
   ```

### 3. Webhook Integration

#### Discord Webhook Setup

1. Create webhook in Discord channel settings
2. Configure webhook appearance and name
3. Copy webhook URL for GitHub configuration

#### GitHub Webhook Configuration

1. Go to repository Settings > Webhooks
2. Add webhook with Discord webhook URL
3. Set content type to `application/json`
4. Select events to trigger webhook:
   - Push events
   - Pull requests
   - Issues
   - Discussions
   - Releases
5. Enable SSL verification

#### Custom Webhook Payload Processing

For advanced formatting, create a custom webhook processor:

```javascript
// webhook-processor.js
const express = require('express');
const bodyParser = require('body-parser');
const axios = require('axios');
const crypto = require('crypto');

const app = express();
const PORT = process.env.PORT || 3000;
const WEBHOOK_SECRET = process.env.WEBHOOK_SECRET;
const DISCORD_WEBHOOK_URL = process.env.DISCORD_WEBHOOK_URL;

// Middleware to verify GitHub webhook signature
function verifyGitHubWebhook(req, res, next) {
  const signature = req.headers['x-hub-signature-256'];
  if (!signature) {
    return res.status(401).send('Signature missing');
  }

  const payload = JSON.stringify(req.body);
  const hmac = crypto.createHmac('sha256', WEBHOOK_SECRET);
  const calculatedSignature = 'sha256=' + hmac.update(payload).digest('hex');
  
  if (crypto.timingSafeEqual(Buffer.from(signature), Buffer.from(calculatedSignature))) {
    next();
  } else {
    res.status(401).send('Invalid signature');
  }
}

app.use(bodyParser.json());

app.post('/github-webhook', verifyGitHubWebhook, async (req, res) => {
  const event = req.headers['x-github-event'];
  const payload = req.body;
  
  try {
    let message;
    
    switch (event) {
      case 'push':
        message = formatPushEvent(payload);
        break;
      case 'pull_request':
        message = formatPullRequestEvent(payload);
        break;
      case 'issues':
        message = formatIssueEvent(payload);
        break;
      case 'discussion':
        message = formatDiscussionEvent(payload);
        break;
      case 'release':
        message = formatReleaseEvent(payload);
        break;
      default:
        message = {
          content: `Received GitHub ${event} event`
        };
    }
    
    await axios.post(DISCORD_WEBHOOK_URL, message);
    res.status(200).send('Webhook processed');
  } catch (error) {
    console.error('Error processing webhook:', error);
    res.status(500).send('Error processing webhook');
  }
});

function formatPushEvent(payload) {
  const branch = payload.ref.replace('refs/heads/', '');
  const commits = payload.commits.slice(0, 5); // Limit to 5 commits
  
  let commitList = '';
  commits.forEach(commit => {
    const shortHash = commit.id.substring(0, 7);
    const message = commit.message.split('\n')[0]; // First line of commit message
    commitList += `• [\`${shortHash}\`](${commit.url}) ${message} - ${commit.author.name}\n`;
  });
  
  if (payload.commits.length > 5) {
    commitList += `• ... and ${payload.commits.length - 5} more commits\n`;
  }
  
  return {
    embeds: [{
      title: `[${payload.repository.name}] ${payload.commits.length} new commit${payload.commits.length !== 1 ? 's' : ''} to ${branch}`,
      url: payload.compare,
      color: 7506394, // Green
      author: {
        name: payload.sender.login,
        url: payload.sender.html_url,
        icon_url: payload.sender.avatar_url
      },
      description: commitList,
      timestamp: new Date().toISOString()
    }]
  };
}

// Implement other formatting functions:
// - formatPullRequestEvent
// - formatIssueEvent
// - formatDiscussionEvent
// - formatReleaseEvent

app.listen(PORT, () => {
  console.log(`Webhook processor listening on port ${PORT}`);
});
```

### 4. Knowledge Base Integration

#### GitHub Activity Monitoring

```javascript
// github-activity-monitor.js
const { Octokit } = require('@octokit/rest');
const knowledgeBase = require('./knowledge-base');

class GitHubActivityMonitor {
  constructor(token, owner, repo) {
    this.octokit = new Octokit({ auth: token });
    this.owner = owner;
    this.repo = repo;
    this.lastChecked = new Date();
  }
  
  async monitorIssues() {
    const { data: issues } = await this.octokit.issues.listForRepo({
      owner: this.owner,
      repo: this.repo,
      state: 'all',
      since: this.lastChecked.toISOString(),
      sort: 'updated',
      direction: 'desc'
    });
    
    for (const issue of issues) {
      // Skip pull requests (they're also returned by the issues API)
      if (issue.pull_request) continue;
      
      // Get issue comments
      const { data: comments } = await this.octokit.issues.listComments({
        owner: this.owner,
        repo: this.repo,
        issue_number: issue.number
      });
      
      // Process issue and comments for knowledge extraction
      await knowledgeBase.processGitHubIssue(issue, comments);
    }
  }
  
  async monitorPullRequests() {
    const { data: prs } = await this.octokit.pulls.list({
      owner: this.owner,
      repo: this.repo,
      state: 'all',
      sort: 'updated',
      direction: 'desc'
    });
    
    for (const pr of prs) {
      if (new Date(pr.updated_at) < this.lastChecked) continue;
      
      // Get PR comments
      const { data: comments } = await this.octokit.pulls.listReviewComments({
        owner: this.owner,
        repo: this.repo,
        pull_number: pr.number
      });
      
      // Process PR and comments for knowledge extraction
      await knowledgeBase.processGitHubPR(pr, comments);
    }
  }
  
  async monitorDiscussions() {
    // Using GraphQL for discussions since REST API doesn't support them
    const query = `
      query($owner:String!, $repo:String!) {
        repository(owner:$owner, name:$repo) {
          discussions(first:10, orderBy:{field:UPDATED_AT, direction:DESC}) {
            nodes {
              id
              title
              body
              createdAt
              updatedAt
              author {
                login
              }
              category {
                name
              }
              comments(first:20) {
                nodes {
                  body
                  author {
                    login
                  }
                  createdAt
                }
              }
            }
          }
        }
      }
    `;
    
    const { repository } = await this.octokit.graphql(query, {
      owner: this.owner,
      repo: this.repo
    });
    
    for (const discussion of repository.discussions.nodes) {
      if (new Date(discussion.updatedAt) < this.lastChecked) continue;
      
      // Process discussion for knowledge extraction
      await knowledgeBase.processGitHubDiscussion(discussion);
    }
  }
  
  async runMonitoring() {
    try {
      await this.monitorIssues();
      await this.monitorPullRequests();
      await this.monitorDiscussions();
      
      this.lastChecked = new Date();
    } catch (error) {
      console.error('Error monitoring GitHub activity:', error);
    }
  }
  
  // Start periodic monitoring
  startMonitoring(intervalMinutes = 15) {
    this.monitoringInterval = setInterval(() => {
      this.runMonitoring();
    }, intervalMinutes * 60 * 1000);
    
    // Run initial monitoring
    this.runMonitoring();
    
    console.log(`GitHub activity monitoring started with ${intervalMinutes} minute interval`);
  }
  
  stopMonitoring() {
    if (this.monitoringInterval) {
      clearInterval(this.monitoringInterval);
      console.log('GitHub activity monitoring stopped');
    }
  }
}

module.exports = GitHubActivityMonitor;
```

## Deployment Instructions

### 1. Discord Bot Deployment

#### Prerequisites
- Node.js 16.x or higher
- npm or yarn
- Discord Bot Token
- GitHub Personal Access Token

#### Setup Steps

1. **Clone the repository**
   ```bash
   git clone https://github.com/APiTJLillo/Anarchy-Inference.git
   cd Anarchy-Inference/community_forum/discord/bot
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Create configuration file**
   Create a `config.json` file:
   ```json
   {
     "token": "YOUR_DISCORD_BOT_TOKEN",
     "clientId": "YOUR_DISCORD_CLIENT_ID",
     "guildId": "YOUR_DISCORD_SERVER_ID",
     "githubToken": "YOUR_GITHUB_TOKEN",
     "owner": "APiTJLillo",
     "repo": "Anarchy-Inference"
   }
   ```

4. **Register slash commands**
   ```bash
   node deploy-commands.js
   ```

5. **Start the bot**
   ```bash
   node bot.js
   ```

### 2. Webhook Processor Deployment

#### Heroku Deployment

1. **Create Heroku app**
   ```bash
   heroku create anarchy-inference-webhook
   ```

2. **Set environment variables**
   ```bash
   heroku config:set WEBHOOK_SECRET=your_github_webhook_secret
   heroku config:set DISCORD_WEBHOOK_URL=your_discord_webhook_url
   ```

3. **Deploy application**
   ```bash
   git subtree push --prefix community_forum/discord/webhook heroku main
   ```

#### GitHub Repository Configuration

1. Go to repository Settings > Webhooks
2. Add webhook with Heroku URL (e.g., `https://anarchy-inference-webhook.herokuapp.com/github-webhook`)
3. Set content type to `application/json`
4. Set secret to match `WEBHOOK_SECRET`
5. Select events to trigger webhook
6. Enable SSL verification

## Integration Testing

### Test Cases

1. **GitHub Bot Notifications**
   - Create test issue
   - Make test commit
   - Open test pull request
   - Verify notifications appear in Discord

2. **Custom Bot Commands**
   - Test `/issue list` command
   - Test `/issue view` command
   - Test `/issue create` command
   - Test other implemented commands

3. **Webhook Processing**
   - Trigger various GitHub events
   - Verify formatted messages in Discord
   - Check error handling for invalid payloads

4. **Knowledge Base Integration**
   - Create discussion with technical content
   - Verify knowledge extraction
   - Test knowledge retrieval via bot commands

## Maintenance Guidelines

### Regular Maintenance

1. **Bot Updates**
   - Keep dependencies updated
   - Monitor Discord API changes
   - Update GitHub API usage as needed

2. **Performance Monitoring**
   - Check bot response times
   - Monitor webhook processing latency
   - Optimize database queries

3. **Error Handling**
   - Set up error logging
   - Create alerts for critical failures
   - Implement automatic recovery procedures

### Troubleshooting Common Issues

1. **Bot Disconnections**
   - Check Discord API status
   - Verify token validity
   - Implement reconnection logic

2. **Missing Notifications**
   - Verify webhook configuration
   - Check event subscription settings
   - Test webhook endpoint manually

3. **Command Failures**
   - Check permission settings
   - Verify GitHub API access
   - Review command implementation

## Next Steps

1. **Implement Custom Bot**
   - Set up development environment
   - Create core command structure
   - Implement GitHub API integration
   - Deploy initial version

2. **Configure Webhooks**
   - Create Discord webhooks
   - Set up GitHub webhook configuration
   - Deploy webhook processor
   - Test notification delivery

3. **Integrate with Knowledge Base**
   - Connect GitHub activity monitor
   - Implement knowledge extraction
   - Create knowledge retrieval commands
   - Test end-to-end functionality
