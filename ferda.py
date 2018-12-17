import requests
from urllib.parse import quote_plus

class TravisSession:

    def __init__(self, token, user_agent):
        self.sess = requests.Session()

        # https://developer.travis-ci.com/gettingstarted
        self.sess.headers = {
            'Travis-API-Version': '3',
            'User-Agent': user_agent,
            'Authorization': f'token {token}',
        }

        def check_for_error(resp, *args, **kwargs):
            resp.raise_for_status()

        self.sess.hooks['response'].append(check_for_error)

    def get(self, endpoint, *args, **kwargs):
        url = f'https://api.travis-ci.org/{endpoint.lstrip("/")}'
        return self.sess.get(url, *args, **kwargs).json()

        sess = TravisSession(
            token='abcdefghij',
            user_agent='example_script.py by alexwlchan')

        sess.get('/repo/wellcometrust%2Fplatform/builds')

    def all_builds(sess, repo_name):
        params = {}
        while True:
            resp = sess.get(f'/repo/{quote_plus(repo_name)}/builds', params=params)
            yield from resp['builds']
            params['offset'] = params.get('offset', 0) + len(resp['builds'])
            if resp['@pagination']['is_last']:
                break

sess = TravisSession(
    token=TOKEN,
    user_agent='track_build_times.py by alexwlchan'
)

for build in all_builds(sess=sess, repo_name='tcr/edit-text'):
    if build['event_type'] != 'cron':
        continue
    print((build['finished_at'], build['duration']))
