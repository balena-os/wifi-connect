import type { JSONSchema7 as JSONSchema } from 'json-schema';
import * as React from 'react';
import type { RenditionUiSchema } from 'rendition';
import { Button, Flex, Form, Heading } from 'rendition';
import type { Network, NetworkInfo } from './App';

const getSchema = (availableNetworks: Network[]): JSONSchema => ({
	type: 'object',
	properties: {
		ssid: {
			title: 'SSID',
			type: 'string',
			default: availableNetworks[0]?.ssid,
			oneOf: availableNetworks.map((network) => ({
				const: network.ssid,
				title: network.ssid,
			})),
		},
		identity: {
			title: 'User',
			type: 'string',
			default: '',
		},
		passphrase: {
			title: 'Passphrase',
			type: 'string',
			default: '',
		},
	},
	required: ['ssid'],
});

const getUiSchema = (isEnterprise: boolean): RenditionUiSchema => ({
	ssid: {
		'ui:placeholder': 'Select SSID',
		'ui:options': {
			emphasized: true,
		},
	},
	identity: {
		'ui:options': {
			emphasized: true,
		},
		'ui:widget': !isEnterprise ? 'hidden' : undefined,
	},
	passphrase: {
		'ui:widget': 'password',
		'ui:options': {
			emphasized: true,
		},
	},
});

const isEnterpriseNetwork = (
	networks: Network[],
	selectedNetworkSsid?: string,
) => {
	return networks.some(
		(network) =>
			network.ssid === selectedNetworkSsid && network.security === 'enterprise',
	);
};

interface NetworkInfoFormProps {
	availableNetworks: Network[];
	onSubmit: (data: NetworkInfo) => void;
	onRefreshNetworks: () => void;
	isRefreshingNetworks: boolean;
}

export const NetworkInfoForm = ({
	availableNetworks,
	onSubmit,
	onRefreshNetworks,
	isRefreshingNetworks,
}: NetworkInfoFormProps) => {
	const [data, setData] = React.useState<NetworkInfo>({});

	const isSelectedNetworkEnterprise = isEnterpriseNetwork(
		availableNetworks,
		data.ssid,
	);

	return (
		<Flex
			flexDirection="column"
			alignItems="center"
			justifyContent="center"
			m={4}
			mt={5}
		>
			<Heading.h3
				align="center"
				mb={4}
				color={'white'}
				style={{ fontWeight: 700 }}
				fontSize={'23px'}
			>
				Hi! Please choose your WiFi from the list
			</Heading.h3>

			<Form
				width={['100%', '80%', '60%', '40%']}
				onFormChange={({ formData }) => {
					setData(formData);
				}}
				onFormSubmit={({ formData }) => {
					onSubmit(formData);
				}}
				value={data}
				schema={getSchema(availableNetworks)}
				uiSchema={getUiSchema(isSelectedNetworkEnterprise)}
				submitButtonProps={{
					width: '60%',
					mx: '20%',
					mt: 3,
					disabled: availableNetworks.length <= 0,
				}}
				submitButtonText={'Connect'}
			/>
			<Button
				type="button"
				width={['60%', '48%', '36%', '24%']}
				mt={2}
				secondary
				outline
				disabled={isRefreshingNetworks}
				onClick={onRefreshNetworks}
			>
				{isRefreshingNetworks ? 'Refreshing...' : 'Refresh WiFi list'}
			</Button>
		</Flex>
	);
};
