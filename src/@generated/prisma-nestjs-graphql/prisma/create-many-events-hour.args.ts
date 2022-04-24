import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_HourCreateManyInput } from '../events-hour/events-hour-create-many.input';

@ArgsType()
export class CreateManyEventsHourArgs {

    @Field(() => [Events_HourCreateManyInput], {nullable:false})
    data!: Array<Events_HourCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
