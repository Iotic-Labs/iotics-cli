classes

- host
- twin
- agent
- user?

Host is class

address is a property
of class Host

DID is a class
key_name is a property of class DID
update_time is a property of class DID

Twin is class
subclass of DID

Agent is a class
subclass of DID

inside_host is a property
of class Twin

controlled_by is a property
of class Twin

ganymede is a Host
address ganymede.iotics.space

did:iotics:agent is an Agent
key_name portal-agent

did:iotics:12345 is a Twin
key_name twin-0
inside_host ganymede
controlled_by did:iotics:agent

questions we will be able to answer:

- which are the twins controlled by an agent
- which are the agents controlling twins in a host
- which are the agents controlling twins in a network
- which twin DIDs were created/updated in the last X days
