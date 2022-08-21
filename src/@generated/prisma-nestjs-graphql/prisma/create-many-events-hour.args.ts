import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourCreateManyInput } from '../events-hour/events-hour-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyEventsHourArgs {

    @Field(() => [Events_HourCreateManyInput], {nullable:false})
    @Type(() => Events_HourCreateManyInput)
    data!: Array<Events_HourCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
