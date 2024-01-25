# lioqing.github.io

This is the repository for my personal website.

## Environment Setup

### Node.js and npm

We use Node.js v18.17.1 and npm v9.6.7, so make sure you have that installed.

You could use [nvm](https://github.com/nvm-sh/nvm) or [nvm-windows](https://github.com/coreybutler/nvm-windows) (Windows is not recommended to install nvm because it does not get native support) to manage your Node.js and npm versions.

Install the Node.js version you want to use, which also installs the npm.

```bash
nvm install <version>
```

Specify the version for you are going to use.

```bash
nvm use <version>
```

To check your Node.js and npm version, run `node --version` and `npm --version` respectively in your terminal.

```bash
node --version
npm --version
```

### Install Dependencies

Rename `typescript-react-template` with your project name in `package.json` and `package-lock.json`, there should be 3 places to change.

Install the dependencies for the project.

```bash
npm install
```

### Lint and Pre-commit

We use [Prettier](https://prettier.io/) and [ESLint](https://eslint.org/) for the coding style and guidelines. The style is then enforced by [Husky](https://typicode.github.io/husky/#/) and [lint-staged](https://github.com/lint-staged/lint-staged).

Finish the environment setup above (especially installing the dependencies with npm) before proceeding.

Install and setup the pre-commit hooks.

```bash
npm run pre-commit
```

To run linting manually (only scans staged files).

```bash
npx lint-staged
```

Remember to stage files again if there are any changes made by the pre-commit hooks or by you.

```bash
git add .
```

### VS Code Settings

You can add a workspace setting to automatically format your code on save using the black formatter.

You need to have the [Prettier - Code formatter](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode) and [Prettier ESLint](https://marketplace.visualstudio.com/items?itemName=rvest.vs-code-prettier-eslint) extensions installed.

Bring up the command palette with `Ctrl+Shift+P`(Windows/Linux) / `Cmd+Shift+P`(Mac) and search for "Preferences: Open Workspace Settings (JSON)".

Then replace the content with the following:

```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "[javascript][typescript][javascriptreact][typescriptreact][json][css][html]": {
    "editor.tabSize": 2
  }
}
```

## Development

### Clone Repository

First clone the repository.

```bash
git clone git@github.com:<username>/<repository>.git
```

**Important**: You may need to setup SSH keys for your GitHub account. See [this guide](https://help.github.com/en/github/authenticating-to-github/connecting-to-github-with-ssh) for more information.

### Checkout Branch

Then checkout the branch you want to work on.

```bash
git checkout <branch>
```

### Committing Changes

Commit your changes to the branch you are working on.

```bash
git add .
git commit -m "Your commit message"
```

Make any changes and stage your files again according to the pre-commit hooks.

### Pushing Changes

Set your branch's upstream branch to be the same branch on the remote repository on GitHub.

```bash
git push -u origin <branch>
```

After the first time you set the upstream branch, you can simply push without specifying the branch.

```bash
git push
```

### Available Scripts

In the project directory, you can run:

#### `npm start`

Runs the app in the development mode.\
Open [http://localhost:3000](http://localhost:3000) to view it in the browser.

The page will reload if you make edits.\
You will also see any lint errors in the console.

#### `npm test`

Launches the test runner in the interactive watch mode.\
See the section about [running tests](https://facebook.github.io/create-react-app/docs/running-tests) for more information.

#### `npm run build`

Builds the app for production to the `build` folder.\
It correctly bundles React in production mode and optimizes the build for the best performance.

The build is minified and the filenames include the hashes.\
The app is ready to be deployed!

See the section about [deployment](https://facebook.github.io/create-react-app/docs/deployment) for more information.
