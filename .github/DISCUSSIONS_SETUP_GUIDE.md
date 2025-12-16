# GitHub Discussions Setup Guide

Complete guide for setting up GitHub Discussions for Better IPTV.

## 🚀 Step 1: Enable Discussions

1. Go to your repository: https://github.com/mewset/better-iptv
2. Click **Settings** (top right)
3. Scroll down to **Features** section
4. Check ✅ **Discussions**
5. Click **Set up discussions** (green button)

## 📁 Step 2: Configure Categories

GitHub will create default categories. **Replace them** with these custom categories:

### Recommended Categories

| Category | Emoji | Description | Format |
|----------|-------|-------------|---------|
| **Announcements** | 📢 | Official project updates and releases | Announcement |
| **General** | 💬 | General discussion about Better IPTV | Open-ended |
| **Ideas** | 💡 | Feature requests and improvements | Open-ended |
| **Q&A** | ❓ | Questions and help from the community | Q&A |
| **Show and Tell** | 🎨 | Share your Better IPTV setup or customizations | Open-ended |
| **Bugs** | 🐛 | Discuss potential bugs before reporting | Open-ended |

### How to Create/Edit Categories

1. Go to **Discussions** tab in your repo
2. Click **Categories** (right sidebar) or go to: `https://github.com/mewset/better-iptv/discussions/categories`
3. Click **New category** or **Edit** (pencil icon)
4. Fill in:
   - **Name**: (e.g., "Bugs")
   - **Description**: (copy from table above)
   - **Emoji**: (copy from table)
   - **Discussion format**: (see table)

### Category Descriptions (Copy-Paste)

**Announcements:**
```
Official project updates, releases, and important news from the maintainers. Only maintainers can post here.
```

**General:**
```
General discussion about Better IPTV. Talk about anything related to the project, IPTV, or share experiences.
```

**Ideas:**
```
Suggest new features, improvements, or changes. Discuss what you'd like to see in Better IPTV.
```

**Q&A:**
```
Ask questions about using Better IPTV. Get help from the community with setup, configuration, or troubleshooting.
```

**Show and Tell:**
```
Share your Better IPTV setup, customizations, or creative uses. Show the community what you've built!
```

**Bugs:**
```
Discuss potential bugs before creating an issue. Verify if something is a bug, get help reproducing issues, or discuss workarounds.
```

## 📌 Step 3: Pin Important Discussions

Create and pin these discussions in relevant categories:

### For "Bugs" Category

1. Go to **Discussions** → **New discussion**
2. Select **Bugs** category
3. Title: `📌 READ FIRST: Before Reporting a Bug`
4. Content: Copy from `.github/discussion-templates/BUG_DISCUSSION_PINNED.md`
5. Post the discussion
6. Click **⋯** (three dots) → **Pin discussion**

### For "Ideas" Category

Title: `📌 Feature Request Guidelines`

Content:
```markdown
# 💡 Before Suggesting a Feature

Thanks for wanting to improve Better IPTV! Please follow these guidelines:

## ✅ Checklist

1. **Search existing ideas**
   - Check [existing discussions](https://github.com/mewset/better-iptv/discussions/categories/ideas) to avoid duplicates
   - Check [feature request issues](https://github.com/mewset/better-iptv/issues?q=is%3Aissue+label%3Aenhancement)

2. **Is this the right place?**
   - **New feature**: Post here in Ideas
   - **Bug fix**: Use [Bug Report](https://github.com/mewset/better-iptv/issues/new?template=bug_report.yml)
   - **Question**: Use [Q&A category](https://github.com/mewset/better-iptv/discussions/categories/q-a)

3. **Describe your idea clearly**
   - What problem does it solve?
   - How would it work?
   - Why is it useful?

## 🎯 Good Feature Requests Include

- **Problem statement**: What challenge are you facing?
- **Proposed solution**: How should it work?
- **Use case**: When would you use this?
- **Examples**: Similar features in other apps (if any)

## ⚡ From Idea to Implementation

Popular ideas may be promoted to official feature requests:
- Create formal [Feature Request Issue](https://github.com/mewset/better-iptv/issues/new?template=feature_request.yml)
- Gets added to the roadmap
- Community can contribute implementation

---

**Let's build Better IPTV together! 🚀**
```

### For "Q&A" Category

Title: `📌 How to Ask Good Questions`

Content:
```markdown
# ❓ Getting Help with Better IPTV

Welcome! Here's how to get the best help from the community:

## ✅ Before Asking

1. **Check the documentation**
   - Read the [README](https://github.com/mewset/better-iptv#readme)
   - Check the [FAQ](https://github.com/mewset/better-iptv#-frequently-asked-questions-faq)

2. **Search existing discussions**
   - Your question might already be answered
   - Use the search: `is:discussion category:Q&A your-topic`

3. **Is this really a question?**
   - **Question**: How to do something, why something happens
   - **Bug**: Something broken → Use [Bug Report](https://github.com/mewset/better-iptv/issues/new?template=bug_report.yml)
   - **Feature request**: Something missing → Use [Ideas](https://github.com/mewset/better-iptv/discussions/categories/ideas)

## 📝 Good Questions Include

- **Clear title**: "How to import M3U from URL?" (not "Help pls")
- **System info**: OS, Better IPTV version
- **What you tried**: Steps you already attempted
- **Expected vs actual**: What you wanted vs what happened
- **Logs** (if error): See log locations in README

## 💡 Tips for Faster Answers

- ✅ **Be specific**: "EPG not updating" instead of "broken"
- ✅ **Include details**: Version, OS, error messages
- ✅ **Show appreciation**: Mark helpful answers ✓
- ✅ **Follow up**: Share if you solved it (helps others!)

---

**The community is here to help! 🤝**
```

## 🎨 Step 4: Customize Discussion Templates

GitHub doesn't support discussion form templates (like issue templates) yet, but you can:

1. **Create discussion category descriptions** (already done above)
2. **Pin helpful discussions** (guides above)
3. **Use labels** for discussions (enable in Settings)

## 🔧 Step 5: Additional Settings

1. Go to **Settings** → **Discussions**
2. Enable:
   - ✅ **Allow users to create discussions**
   - ✅ **Enable reactions**
   - ✅ **Enable voting** (for Ideas category)
3. Consider:
   - Require approval for first-time contributors
   - Enable discussion labels

## 📊 Step 6: Add Discussion Links

Update your README.md to link to Discussions:

```markdown
## 💬 Community & Support

- 💡 [Feature Ideas](https://github.com/mewset/better-iptv/discussions/categories/ideas) - Suggest new features
- ❓ [Q&A](https://github.com/mewset/better-iptv/discussions/categories/q-a) - Ask questions
- 🐛 [Bug Discussions](https://github.com/mewset/better-iptv/discussions/categories/bugs) - Discuss potential issues
- 🎨 [Show and Tell](https://github.com/mewset/better-iptv/discussions/categories/show-and-tell) - Share your setup
- 📢 [Announcements](https://github.com/mewset/better-iptv/discussions/categories/announcements) - Project updates
```

## ✅ Completion Checklist

- [ ] Enable Discussions in repository settings
- [ ] Create/customize 6 categories (see table above)
- [ ] Pin "READ FIRST" discussion in Bugs category
- [ ] Pin "Feature Request Guidelines" in Ideas category
- [ ] Pin "How to Ask Questions" in Q&A category
- [ ] Update README.md with discussion links
- [ ] Post welcome announcement in Announcements category

## 🎉 First Announcement

After setup, post your first announcement:

**Title:** `🎉 Welcome to Better IPTV Discussions!`

**Content:**
```markdown
# Welcome to the Better IPTV Community! 🎉

GitHub Discussions is now live! This is your space to:

- 💡 Suggest features
- ❓ Ask questions
- 🐛 Discuss bugs
- 🎨 Share your setup
- 💬 Connect with other users

## 📌 Important Links

- [Bug Reports](https://github.com/mewset/better-iptv/issues/new?template=bug_report.yml) - For confirmed bugs
- [Feature Requests](https://github.com/mewset/better-iptv/issues/new?template=feature_request.yml) - For formal feature proposals
- [Documentation](https://github.com/mewset/better-iptv#readme) - README and FAQ

## 🤝 Community Guidelines

- Be respectful and constructive
- Search before posting
- Mark helpful answers ✓
- Share knowledge and help others

Let's build an amazing IPTV community together! 🚀

---

*P.S. If you enjoy Better IPTV, consider [supporting the project on Ko-fi](https://ko-fi.com/R6R21I53PD)!* ☕
```

---

**Last Updated:** 2025-12-16
