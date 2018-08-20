$('.chosen-select').chosen({disable_search_threshold: 10});

$(document).on('change', '.custom-file-input', function () {
	let fileName = $(this).val().replace(/\\/g, '/').replace(/.*\//, '');
	$(this).parent('.custom-file').find('.custom-file-label').text(fileName);
});

$(function(){
	/////////////////////////////////////////////////////////////////////////
	// Networks view
	//

	var networks = undefined;

	$.get('/networks', function(data){
		$('#wifi-networks-list').empty();

		networks = JSON.parse(data);
		$.each(networks, function(i, val){
			/*
			Reference view:

			<a href="#" class="list-group-item list-group-item-action">
			  <h5 class="d-flex">
				<span class="flex-grow-1">SSID</span>
				<i class="fas fa-check mr-3 light-icon"></i>
				<i class="fas fa-lock mr-3 light-icon"></i>
				<i class="fas fa-wifi strength-10"></i>
			  </h5>
			</a>
			*/

			let blackness = Math.round((val.strength - 1) / 10);
			let strengthClass = 'strength-' + blackness;

			let h5 = $('<h5 class="d-flex">')
				.append(
					$('<span class="flex-grow-1">')
						.text(val.ssid)
				);

			if (val.active === true) {
				h5.append($('<i class="fas fa-check mr-3 light-icon"></i>'));
			}

			if (val.security !== 'none') {
				h5.append($('<i class="fas fa-lock mr-3 light-icon"></i>'));
			}

			h5.append(
				$('<i class="fas fa-wifi"></i>')
					.addClass(strengthClass)
			);

			let link = $('<a href="#" class="list-group-item list-group-item-action">')
				.append(h5);

			link.data('security', val.security);

			$('#wifi-networks-list').append(link);
		});
	});

	$('#wifi-networks-list').on('click', 'a', function (e) {
		e.preventDefault();

		$('#wifi-networks').hide();
		$('#settings').show();

		let security = $(this).data('security');

		$('#security-select').val(security).trigger('chosen:updated').change();
	})

	/////////////////////////////////////////////////////////////////////////
	// Security view
	//

	$('#security-select').chosen().change(function (event) {
		$('option:selected', this).tab('show');
	});

	$('#eap-authentication').chosen().change(function (event) {
		$('option:selected', this).tab('show');

		if($(this).val() === 'tls') {
			$('#eap-username-password-pane').hide();
		} else {
			$('#eap-username-password-pane').show();
		}
	});

	/////////////////////////////////////////////////////////////////////////
	// Dynamic addresses UI
	//

	function hasEmptyAddress(addresses) {
		let result = false;
		addresses.find('.address-input').each(function() {
			if(!$(this).val()) {
				result = true;
				return;
			}
		});
		return result;
	}

	$('.addresses-list').on('blur', '.address-input', function (e) {
		let addresses = $(this).closest('.addresses-list');
		if(hasEmptyAddress(addresses)) {
			return;
		}

		let line = $($('.line', addresses)[0]).clone();
		$('input', line).val('');

		addresses.append(line[0]);
	});

	$('.addresses-list').on('click', '.remove', function () {
		if($(this).closest('.addresses-list').find('.address-input').length < 2) {
			return;
		}

		$(this).closest('.line').remove();
	});

	/////////////////////////////////////////////////////////////////////////
	// Connect form submit
	//

	$('#connect-form').submit(function(e) {
		console.log(
			$('form').serializeJSON({checkboxUncheckedValue: 'false', parseBooleans: true})
		);

		e.preventDefault();
	});
});
