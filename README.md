# Simple tool to create stacked PRs on Github using `jj`

## How it works

When running `stack-prs` without any arguments, it will get all changes between `trunk()` and `@` which are `mine()`
and present them in a text file in my `$EDITOR`.
That text file will look like this:

```
# The following file represents your stack in the order it will applied, top to bottom.
# The first collumn can be one of:
# * "skip"" or "s": to skip this change entirely (can also just delete the line)
# * "create-pr" or "pr": to create the PR based on an already existing bookmark
# * "bookmark" or "b": to create a named bookmark to then use for the PR
# the other columns are:
# * the change ID
# * the change description
# * if present, the bookmark
bookmark,pzkkouuwrxkrpoxqknztyqkpwtuqzqmz,Pass the architecture down to the Helm chart on render,enops-2222
bookmark,utounnzrstvosknnorusyryvwywwqlwp,Detect arch with uname,enops-1111
pr,rzpwqyytylqxowwlmywkpvpyqwlzuzyy,Create multi arch image,enops-1234
s,nsqzmntuqwqulqnxnwnxkypqtqklstov,Use alpha releaser to relaese releaser,
s,tvqnnqqmvtmsqsvwootxswqvrowwxnrs,Empty commit to re-trigger CI,arm-detection*
s,sytpssmnwpswywzzxuwxqzunkooprtlu,,
```

When this file is saved, `create-pr` will do the following:
* create and push a bookmark called `enops-2222` for the revision `pzkkouuwrxkrpoxqknztyqkpwtuqzqmz` and create a PR against main for it
* create and push a bookmark called `neops-1111` for revision `utounnzrstvosknnorusyryvwywwqlwp` and create a PR against the branch `enops-2222` for it
* create a PR from the existing branch `enops-12345` against branch `enops-1111`
* skip the last 3 changes


