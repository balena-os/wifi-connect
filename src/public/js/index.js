$(function(){
	$.get("/ssids", function(data){
		if(data.length == 0){
			$('.before-submit').hide();
			$('#no-networks-message').removeClass('hidden');
		} else {
			$.each(data, function(i, val){
				$("#ssid-select").append("<option value='" + val.ssid + "'>" + val.ssid + "</option>");
			});
		}
	})

	$('#connect-form').submit(function(ev){
		$.post('/connect', $('#connect-form').serialize(), function(data){
			$('.before-submit').hide();
			$('#submit-message').removeClass('hidden');
		});
		ev.preventDefault();
	});
});
