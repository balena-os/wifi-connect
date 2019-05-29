/**
 * SSIDセレクトボックスおよび接続ボタンの振舞いを表す。
 *
 * <p> jQueryのreadyイベント関数として記述している。
 * 事前にjQueryライブラリを読込む必要がある。
 * Javascriptにおける即時関数ではない。
 *
 * <p> {@code ui/index.html} で利用している。
 */
$(function(){
    /**
     * Json形式で所有しているSSIDデータ
     */
	var networks = undefined;

    /**
     * WPA-Enterprise方式の場合はユーザ名テキストエリアを表示する。
     *
     * <p> SSIDセレクトボックスにおいて、WPA-Enterprise方式のSSIDを選択した場合、
     * SSIDセレクトボックスとパスフレーズテキストエリアの間に、
     * ユーザ名テキストエリアを表示する。
     *
     * <p> {@code #ssid-select} の振舞いで利用する。
     */
	function showHideEnterpriseSettings() {
		var security = $(this).find(':selected').attr('data-security');
		if(security === 'enterprise') {
			$('#identity-group').show();
		} else {
			$('#identity-group').hide();
		}
	}

    /**
     * SSIDセレクトボックスにおけるSSID選択時の動作を設定する。
     */
	$('#ssid-select').change(showHideEnterpriseSettings);

    /**
     * SSID選択肢リストを作成してセレクトボックスに格納する。
     *
     * <p> {@code <Gateway_IP_address>/network} 接続時のJson形式のレスポンスデータをパースし、
     * SSID選択肢リストとして成型する。
     * 成型したSSID選択肢リストを {@code #ssid-select} に格納する。
     *
     * <p> 格納する際に、 {@link #showHideEnterpriseSettings()} を呼出す。
     */
	$.get("/networks", function(data){
		if(data.length === 0){
			$('.before-submit').hide();
			$('#no-networks-message').removeClass('hidden');
		} else {
			networks = JSON.parse(data);
			$.each(networks, function(i, val){
                // 空SSID（シークレットSSID）をリストから除去
                if((val.ssid !== '') && ($('#ssid-select option[val="' + val.ssid + '"]').length === 0)){
                    $('#ssid-select').append(
                        $('<option>')
                            .text(val.ssid)
                            .attr('val', val.ssid)
                            .attr('data-security', val.security)
                    );
                };
			});

			jQuery.proxy(showHideEnterpriseSettings, $('#ssid-select'))();
		}
	});

    /**
     * 接続ボタン押下時の挙動を設定する。
     */
	$('#connect-form').submit(function(ev){
		$.post('/connect', $('#connect-form').serialize(), function(data){
			$('.before-submit').hide();
			$('#submit-message').removeClass('hidden');
		});
		ev.preventDefault();
	});
});
