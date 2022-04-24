import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_DayCreateManyInput } from '../events-day/events-day-create-many.input';

@ArgsType()
export class CreateManyEventsDayArgs {

    @Field(() => [Events_DayCreateManyInput], {nullable:false})
    data!: Array<Events_DayCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
