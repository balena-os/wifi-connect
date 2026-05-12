import React from 'react';
import { Navbar, Provider, Container } from 'rendition';
import { NetworkInfoForm } from './NetworkInfoForm';
import { Notifications } from './Notifications';
import { createGlobalStyle } from 'styled-components';

const GlobalStyle = createGlobalStyle`
	html {
		overscroll-behavior-y: none;
	}

	body {
		margin: 0;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
			'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
			sans-serif;
		background: var(--main);
		color: var(--txt);
		overscroll-behavior-y: none;
		-webkit-font-smoothing: antialiased;
		-moz-osx-font-smoothing: grayscale;
	}

	code {
		font-family: source-code-pro, Menlo, Monaco, Consolas, 'Courier New', monospace;
	}

	form label {
		font-weight: 700;
	}

	#root_ssid {
		border: solid var(--lighter) !important;
	}

	form input {
		border: solid var(--lighter) !important;
	}

	#root_ssid__input {
		border: none !important;
	}

	form input:focus,
	form select:focus,
	form textarea:focus {
		box-shadow: none !important;
	}

	#root_ssid__input {
		color: var(--txt) !important;
	}

	.StyledIcon-sc-ofa7kd-0 path {
		stroke: var(--txt) !important;
	}

	#root_ssid__select-drop {
		background-color:  var(--darker) !important;
		color: white !important;
		max-height: 60vh !important;
		overflow-y: auto !important;
		overscroll-behavior-y: contain;
		-webkit-overflow-scrolling: touch;
		touch-action: pan-y;
	}

	#root_ssid__select-drop > div,
	#root_ssid__select-drop [role="listbox"] {
		overscroll-behavior-y: contain;
		-webkit-overflow-scrolling: touch;
	}

	#root_ssid__select-drop [role="option"][aria-selected="true"] {
		background-color: var(--darker) !important;
		color: var(--primary) !important;
	}

	#root_ssid__select-drop [role="option"]:hover {
		background-color: var(--primary) !important;
		color: white !important;
	}
`;

export interface NetworkInfo {
	ssid?: string;
	identity?: string;
	passphrase?: string;
}

export interface Network {
	ssid: string;
	security: string;
}

const App = () => {
	const [attemptedConnect, setAttemptedConnect] = React.useState(false);
	const [isFetchingNetworks, setIsFetchingNetworks] = React.useState(true);
	const [isRefreshingNetworks, setIsRefreshingNetworks] = React.useState(false);
	const [error, setError] = React.useState('');
	const [availableNetworks, setAvailableNetworks] = React.useState<Network[]>(
		[],
	);

	React.useEffect(() => {
		fetch('/networks', { cache: 'no-store' })
			.then((data) => {
				if (data.status !== 200) {
					throw new Error(data.statusText);
				}

				return data.json();
			})
			.then(setAvailableNetworks)
			.catch((e: Error) => {
				setError(`Failed to fetch available networks. ${e.message || e}`);
			})
			.finally(() => {
				setIsFetchingNetworks(false);
			});
	}, []);

	const onConnect = (data: NetworkInfo) => {
		setAttemptedConnect(true);
		setError('');

		fetch('/connect', {
			method: 'POST',
			body: JSON.stringify(data),
			headers: {
				'Content-Type': 'application/json',
			},
		})
			.then((resp) => {
				if (resp.status !== 200) {
					throw new Error(resp.statusText);
				}
			})
			.catch((e: Error) => {
				setError(`Failed to connect to the network. ${e.message || e}`);
			});
	};

	const onRefreshNetworks = () => {
		setIsRefreshingNetworks(true);
		setError('');

		fetch('/networks/refresh', {
			method: 'POST',
		})
			.then((resp) => {
				if (resp.status !== 202 && resp.status !== 200) {
					throw new Error(resp.statusText);
				}
			})
			.catch((e: Error) => {
				setIsRefreshingNetworks(false);
				setError(`Failed to refresh available networks. ${e.message || e}`);
			});
	};

	const customTheme = {
		colors: {
			primary: {
				main: '#FF8D28',
				light: '#FFA352',
				dark: '#E67610',
			},
			secondary: {
				main: '#FFFFFF',
				light: '#FFFFFF',
				dark: '#FFFFFF',
			},
			tertiary: {
				main: '#FF8D28',
				light: '#FF8D28',
				dark: '#FF8D28',
			},
			// quartenary: {
			// 	main: '#FF8D28',
			// 	light: '#FF8D28',
			// 	dark: '#FF8D28',
			// },
			text: {
				main: '#FFFFFF',
				light: '#FFFFFF',
				dark: '#FFFFFF',
			},
			neutral: {
				main: '#FFFFFF',
				light: '#FFFFFF',
				dark: '#FFFFFF',
			},
			aaaaaaaa: {
				main: '#FF8D28',
				light: '#FF8D28',
				dark: '#FF8D28',
			},
		},
	};
	return (
		<Provider theme={customTheme}>
			<GlobalStyle />
			{/* <Navbar brand={<img src={logo} style={{ height: 30 }} alt="logo" />} /> */}
			<Navbar
				brand={
					<div
						style={{ fontWeight: '700', fontSize: '24px', fontFamily: 'K2D' }}
					>
						PlayControl
					</div>
				}
				style={{
					background: 'var(--header)',
				}}
			></Navbar>

			<Container>
				<Notifications
					attemptedConnect={attemptedConnect}
					isRefreshingNetworks={isRefreshingNetworks}
					hasAvailableNetworks={
						isFetchingNetworks || availableNetworks.length > 0
					}
					error={error}
				/>
				<NetworkInfoForm
					availableNetworks={availableNetworks}
					// availableNetworks={[
					// 	{ ssid: 'Home WiFi', security: 'wpa2' },
					// 	{ ssid: 'Office Network', security: 'wpa2' },
					// 	{ ssid: 'Guest', security: 'open' },
					// 	{ ssid: 'Enterprise Net', security: 'enterprise' },
					// ]}
					onSubmit={onConnect}
					onRefreshNetworks={onRefreshNetworks}
					isRefreshingNetworks={isRefreshingNetworks}
				/>
			</Container>
		</Provider>
	);
};

export default App;
