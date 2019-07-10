package app

import (
	"os"
	"os/signal"

	"github.com/SierraSoftworks/git-tool/internal/pkg/autocomplete"
	"github.com/SierraSoftworks/git-tool/internal/pkg/di"

	"github.com/pkg/errors"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli"
)

var openAppCommand = cli.Command{
	Name: "open",
	Aliases: []string{
		"run",
		"o",
	},
	Usage:     "Opens the requested repository in a specific command.",
	ArgsUsage: "[app] [repo]",
	Flags:     []cli.Flag{},
	Action: func(c *cli.Context) error {
		args := c.Args()

		app := di.GetConfig().GetApp(c.Args().First())
		if app == nil {
			app = di.GetConfig().GetDefaultApp()
		} else {
			args = cli.Args(c.Args().Tail())
		}

		if app == nil && c.NArg() > 0 {
			return errors.Errorf("no app called %s in your config", c.Args().First())
		} else if app == nil {
			return errors.Errorf("no apps in your config")
		}

		logrus.WithField("app", app.Name()).Debug("Found matching app configuration")

		r, err := di.GetMapper().GetBestRepo(args.First())
		if err != nil {
			return err
		}

		if !r.Exists() {
			init := di.GetInitializer()

			err := init.Clone(r)
			if err != nil {
				return errors.New("repository doesn't exist yet, use 'new' to create it")
			}

			logrus.Info("Cloned repository to your local filesystem")
		}

		cmd, err := app.GetCmd(r)
		if err != nil {
			return err
		}

		cmd.Stdin = os.Stdin
		cmd.Stderr = os.Stderr
		cmd.Stdout = os.Stdout

		sig := make(chan os.Signal, 1)
		signal.Notify(sig)

		go func() {
			for s := range sig {
				if cmd.Process != nil && cmd.ProcessState != nil && !cmd.ProcessState.Exited() {
					cmd.Process.Signal(s)
				}
			}
		}()

		defer func() {
			// Shutdown signal forwarding for our process
			signal.Stop(sig)
			close(sig)
		}()

		return cmd.Run()
	},
	BashComplete: func(c *cli.Context) {
		cmp := autocomplete.NewCompleter(c.GlobalString("bash-completion-filter"))

		if c.NArg() == 0 {
			cmp.Apps()
		}

		if app := di.GetConfig().GetApp(c.Args().First()); app != nil {
			cmp.RepoAliases()
			cmp.DefaultServiceRepos()
			cmp.AllServiceRepos()
		}
	},
}