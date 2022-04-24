import { Injectable } from '@nestjs/common';
import { EventsCreateInput} from '../@generated/prisma-nestjs-graphql/events/events-create.input';
import { EventsUpdateInput} from '../@generated/prisma-nestjs-graphql/events/events-update.input';
import { EventsWhereUniqueInput} from '../@generated/prisma-nestjs-graphql/events/events-where-unique.input';
import { PrismaService } from '../../prisma/prisma.service';

@Injectable()
export class EventsService {
  constructor(private prisma: PrismaService) {}

  create(createEventInput: EventsCreateInput) {
    return this.prisma.events.create({
      data: createEventInput,
    });
  }

  findAll() {
    return this.prisma.events.findMany();
  }

  findOne(eventsWhereUniqueInput: EventsWhereUniqueInput) {
    return this.prisma.events.findUnique({
      where: eventsWhereUniqueInput,
    });
  }

  update(
    eventsWhereUniqueInput: EventsWhereUniqueInput,
    updateEventInput: EventsUpdateInput
  ) {
    return this.prisma.events.update({
      where: eventsWhereUniqueInput,
      data: updateEventInput,
    });
  }

  remove(eventsWhereUniqueInput: EventsWhereUniqueInput) {
    return this.prisma.events.delete({
      where: eventsWhereUniqueInput,
    });
  }
}
