# PR Preview Deployment Setup

This repository now supports automatic preview deployments for Pull Requests using GitHub Actions.

## How it works

When a Pull Request is opened or updated:

1. **Automatic Build**: The PR preview workflow automatically builds the application with the latest changes
2. **Branch Creation**: A preview branch named `preview/pr-{number}` is created with the built application  
3. **Preview Link**: A comment is automatically added to the PR with a link to view the preview
4. **Auto Updates**: The preview is updated whenever new commits are pushed to the PR
5. **Cleanup**: When the PR is closed, the preview branch is automatically deleted

## Preview URLs

Preview deployments will be available at:
```
https://code-for-groningen.github.io/BetterVIZIER/preview/pr-{number}/
```

Where `{number}` is the Pull Request number.

## Initial Setup Required

**For Repository Maintainers**: The first time a preview is created, you may need to enable GitHub Pages to serve from multiple branches:

1. Go to **Settings** → **Pages** in the repository
2. Ensure GitHub Pages is enabled 
3. The source should be set to "Deploy from a branch"
4. Individual preview branches will be accessible via the standard GitHub Pages subdirectory structure

## Files Added/Modified

- `.github/workflows/deploy-preview.yml` - Main preview deployment workflow
- `.github/workflows/cleanup-preview.yml` - Cleanup workflow for closed PRs  
- `fitch-proof/packages/app/vite.config.ts` - Updated to support dynamic base paths

## Benefits

- **🚀 Faster Reviews**: Reviewers can instantly see and test changes
- **🐛 Early Bug Detection**: Issues can be spotted before merging
- **🔗 Easy Sharing**: Preview links make it easy to share work-in-progress
- **♻️ Clean**: Automatic cleanup when PRs are closed