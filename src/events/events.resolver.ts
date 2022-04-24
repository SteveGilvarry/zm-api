import { Resolver, Query, Mutation, Args } from '@nestjs/graphql';
import { EventsService } from './events.service';
import { EventsCreateInput} from '../@generated/prisma-nestjs-graphql/events/events-create.input';
import { EventsUpdateInput} from '../@generated/prisma-nestjs-graphql/events/events-update.input';
import { EventsWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/events/events-where-unique.input';

@Resolver('Event')
export class EventsResolver {
  constructor(private readonly eventsService: EventsService) {}

  @Mutation('createEvent')
  async create(
    @Args('createEventInput') createEventInput: EventsCreateInput,
  ){
    const created = await this.eventsService.create(createEventInput);
  }

  @Query('events')
  findAll() {
    return this.eventsService.findAll();
  }

  @Query('event')
  findOne(@Args('id') Id: number) {
    return this.eventsService.findOne( { Id });
  }

  @Mutation('updateEvent')
  update(
    @Args( 'id') Id: number,
    @Args('updateEventInput') updateEventInput: EventsUpdateInput) {
    return this.eventsService.update({  Id }, updateEventInput);
  }

  @Mutation('removeEvent')
  remove(@Args('id') Id: number) {
    return this.eventsService.remove({ Id } );
  }
}
