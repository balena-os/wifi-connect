import { Txt, Alert } from 'rendition';

export const Notifications = ({
	hasAvailableNetworks,
	attemptedConnect,
	error,
}: {
	hasAvailableNetworks: boolean;
	attemptedConnect: boolean;
	error: string;
}) => {
	return (
		<>
			{attemptedConnect && (
				<Alert m={2} info>
					<Txt.span>Applying changes... </Txt.span>
					<Txt.span>
						Your device will soon be online. If connection is unsuccessful, the
						Access Point will be back up in a few minutes, and reloading this
						page will allow you to try again.
					</Txt.span>
				</Alert>
			)}
			{!hasAvailableNetworks && (
				<Alert m={2} warning>
					<Txt.span>No wifi networks available.&nbsp;</Txt.span>
					<Txt.span>
						Please ensure there is a network within range and reboot the device.
					</Txt.span>
				</Alert>
			)}
			{!!error && (
				<Alert m={2} danger>
					<Txt.span>{error}</Txt.span>
				</Alert>
			)}
		</>
	);
};
