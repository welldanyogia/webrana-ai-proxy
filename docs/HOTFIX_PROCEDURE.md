# Hotfix Procedure

## Overview

This document outlines the procedure for deploying critical bug fixes to production during launch week. The hotfix process is designed to deploy fixes within 30 minutes while maintaining code quality.

**Requirements:** 7.1, 7.2, 7.3, 7.5

## When to Use Hotfix

Use the hotfix process when:
- ✅ Critical bug affecting user experience
- ✅ Security vulnerability discovered
- ✅ Payment/billing issues
- ✅ API endpoint returning errors
- ✅ Data integrity issues

Do NOT use hotfix for:
- ❌ Feature additions
- ❌ Non-critical UI improvements
- ❌ Performance optimizations (unless critical)
- ❌ Documentation updates

## Hotfix Process

### 1. Create Hotfix Branch (2 min)

```bash
# From main branch
git checkout main
git pull origin main
git checkout -b hotfix/issue-description
```

### 2. Implement Fix (10-15 min)

- Make minimal changes to fix the issue
- Add unit test if possible
- Do NOT refactor unrelated code

### 3. Local Testing (5 min)

```bash
# Backend
cd infrastructure/docker
docker compose run --rm backend-dev cargo test

# Frontend
cd frontend
npm test -- --run
```

### 4. Create Pull Request (2 min)

- Title: `[HOTFIX] Brief description`
- Description: Include issue link, root cause, fix summary
- Label: `hotfix`, `urgent`
- Request review from 1 team member

### 5. Code Review (5-10 min)

**Reviewer checklist:**
- [ ] Fix addresses the issue
- [ ] No unrelated changes
- [ ] Tests pass
- [ ] No security concerns

### 6. Merge and Deploy (5 min)

Once approved:
1. Merge to `hotfix/*` branch
2. GitHub Actions automatically deploys
3. Monitor deployment in Slack

### 7. Post-Deployment (5 min)

- [ ] Verify fix in production
- [ ] Update status page if applicable
- [ ] Notify team in Slack
- [ ] Create follow-up ticket if needed

## Rollback Procedure

If hotfix causes regression:

### Automatic Rollback (< 5 min)

```bash
# Via kubectl
kubectl rollout undo deployment/webrana-backend -n webrana
kubectl rollout undo deployment/webrana-frontend -n webrana
```

### Manual Rollback

1. Go to GitHub Actions
2. Find last successful deployment
3. Re-run deployment job with previous commit

## Deployment Log

All hotfix deployments are logged with:
- Timestamp (UTC)
- Commit hash
- Deployer name
- Reason/issue link

View logs:
```bash
kubectl logs -l app=webrana-backend -n webrana --since=1h
```

## Communication

### During Hotfix

1. Post in #engineering Slack channel
2. Update status page if user-facing
3. Notify on-call team member

### After Hotfix

1. Send summary to team
2. Schedule post-mortem if needed
3. Update documentation if process changed

## Emergency Contacts

| Role | Contact |
|------|---------|
| On-call Engineer | Check PagerDuty |
| DevOps Lead | @devops-lead |
| Product Manager | @pm |

## Hotfix Checklist

```markdown
## Pre-Deployment
- [ ] Issue identified and documented
- [ ] Hotfix branch created from main
- [ ] Fix implemented with minimal changes
- [ ] Local tests pass
- [ ] PR created with proper labels
- [ ] Code review approved (1 reviewer)

## Deployment
- [ ] Merged to hotfix branch
- [ ] GitHub Actions triggered
- [ ] Deployment successful
- [ ] Health checks pass

## Post-Deployment
- [ ] Fix verified in production
- [ ] Status page updated (if needed)
- [ ] Team notified
- [ ] Deployment logged
- [ ] Follow-up ticket created (if needed)
```

## Timeline Target

| Step | Target Time |
|------|-------------|
| Create branch | 2 min |
| Implement fix | 15 min |
| Local testing | 5 min |
| PR & Review | 10 min |
| Deploy | 5 min |
| Verify | 3 min |
| **Total** | **< 30 min** |
