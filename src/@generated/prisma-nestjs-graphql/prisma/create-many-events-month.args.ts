import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthCreateManyInput } from '../events-month/events-month-create-many.input';

@ArgsType()
export class CreateManyEventsMonthArgs {

    @Field(() => [Events_MonthCreateManyInput], {nullable:false})
    data!: Array<Events_MonthCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
