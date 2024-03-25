import * as React from 'react';
import { Txt, Alert } from 'rendition';
import { T } from './Localize';

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
					<Txt.span>{T('applying_changes')}</Txt.span>
					<Txt.span>{T('device_will_soon_be_online')}</Txt.span>
				</Alert>
			)}
			{!hasAvailableNetworks && (
				<Alert m={2} warning>
					<Txt.span>{T('no_wifi_network_available')}&nbsp;</Txt.span>
					<Txt.span>{T('ensure_wifi_in_range')}</Txt.span>
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
