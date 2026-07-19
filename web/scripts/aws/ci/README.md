# CI web-deploy role (GitHub OIDC)

One-time AWS + GitHub setup that lets the `.github/workflows/web-deploy.yml`
manual workflow deploy the site with **no long-lived AWS keys** — GitHub Actions
assumes a least-privilege IAM role via OIDC.

Account `REDACTED-ACCOUNT-ID`, region `us-east-1`, bucket `naba.ysapp.net`, CloudFront
`REDACTED-CF-DISTRIBUTION`. The two policy documents here are the source of truth for the
role; re-apply them if the role drifts.

## 1. Create the GitHub OIDC provider (once per account)

Skip if `aws iam list-open-id-connect-providers` already lists
`token.actions.githubusercontent.com`.

```bash
aws iam create-open-id-connect-provider \
  --url https://token.actions.githubusercontent.com \
  --client-id-list sts.amazonaws.com \
  --thumbprint-list 6938fd4d98bab03faadb97b34396831e3780aea1
```

(AWS validates GitHub's OIDC tokens against its own trust store now; the
thumbprint is required by the API but effectively ignored.)

## 2. Create the role and attach the least-privilege policy

```bash
aws iam create-role \
  --role-name naba-web-deploy \
  --assume-role-policy-document file://web/scripts/aws/ci/trust-policy.json

aws iam put-role-policy \
  --role-name naba-web-deploy \
  --policy-name naba-web-deploy \
  --policy-document file://web/scripts/aws/ci/deploy-policy.json
```

Role ARN: `arn:aws:iam::REDACTED-ACCOUNT-ID:role/naba-web-deploy`

- `trust-policy.json` — who may assume the role: GitHub Actions running in
  `repo:dixson3/naba` (any ref/workflow). Tighten the `sub` to a GitHub
  Environment (`repo:dixson3/naba:environment:production`) if you later gate the
  deploy behind an environment with required reviewers.
- `deploy-policy.json` — what the role may do: S3 write/delete on the site
  bucket + CloudFront invalidation on the one distribution. Nothing else.

## 3. Set the GitHub repo variables + secret

```bash
gh variable set AWS_DEPLOY_ROLE_ARN --body "arn:aws:iam::REDACTED-ACCOUNT-ID:role/naba-web-deploy"
gh variable set CF_DISTRIBUTION     --body "REDACTED-CF-DISTRIBUTION"
gh secret   set NABA_GA_MEASUREMENT_ID   # paste the GA4 id (optional; unset => no analytics)
```

## 4. Deploy

Actions tab → **Deploy web** → **Run workflow**. It builds the site with
production settings, syncs to S3, and invalidates CloudFront.
