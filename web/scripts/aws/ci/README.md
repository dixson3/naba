# CI web-deploy role (GitHub OIDC)

One-time AWS + GitHub setup that lets the `.github/workflows/web-deploy.yml`
manual workflow deploy the site with **no long-lived AWS keys** — GitHub Actions
assumes a least-privilege IAM role via OIDC.

Region `us-east-1`, bucket `naba.ysapp.net`. **Account-specific values (AWS
account id, CloudFront distribution id) are never hardcoded here** — the policy
documents are `envsubst` templates (`*.json.tmpl`), and the concrete ids come
from the environment (`aws sts get-caller-identity` and the gitignored
`web/aws-config.mk` written by `provision_aws.sh`), matching the convention in
`provision_aws.sh`.

## 0. Resolve the account-specific values into the environment

```bash
export AWS_ACCOUNT_ID="$(aws sts get-caller-identity --query Account --output text)"
export CF_DISTRIBUTION="$(sed -n 's/^CF_DISTRIBUTION=//p' web/aws-config.mk)"
[ -n "$AWS_ACCOUNT_ID" ] && [ -n "$CF_DISTRIBUTION" ] || { echo "missing account id / distribution id"; }
```

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

## 2. Render the templates and create the role

```bash
envsubst < web/scripts/aws/ci/trust-policy.json.tmpl  > /tmp/naba-trust.json
envsubst < web/scripts/aws/ci/deploy-policy.json.tmpl > /tmp/naba-deploy.json

aws iam create-role \
  --role-name naba-web-deploy \
  --assume-role-policy-document file:///tmp/naba-trust.json

aws iam put-role-policy \
  --role-name naba-web-deploy \
  --policy-name naba-web-deploy \
  --policy-document file:///tmp/naba-deploy.json
```

Role ARN: `arn:aws:iam::${AWS_ACCOUNT_ID}:role/naba-web-deploy`

- `trust-policy.json.tmpl` — who may assume the role: GitHub Actions running in
  `repo:dixson3/naba` (any ref/workflow). Tighten the `sub` to a GitHub
  Environment (`repo:dixson3/naba:environment:production`) if you later gate the
  deploy behind an environment with required reviewers.
- `deploy-policy.json.tmpl` — what the role may do: S3 write/delete on the site
  bucket + CloudFront invalidation on the one distribution. Nothing else.

## 3. Set the GitHub repo variables + secret

```bash
gh variable set AWS_DEPLOY_ROLE_ARN --body "arn:aws:iam::${AWS_ACCOUNT_ID}:role/naba-web-deploy"
gh variable set CF_DISTRIBUTION     --body "${CF_DISTRIBUTION}"
gh secret   set NABA_GA_MEASUREMENT_ID   # paste the GA4 id (optional; unset => no analytics)
```

## 4. Deploy

Actions tab → **Deploy web** → **Run workflow**. It builds the site with
production settings, syncs to S3, and invalidates CloudFront.
