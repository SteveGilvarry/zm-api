import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { Events_MonthCreateManyInput } from '../events-month/events-month-create-many.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateManyEventsMonthArgs {

    @Field(() => [Events_MonthCreateManyInput], {nullable:false})
    @Type(() => Events_MonthCreateManyInput)
    data!: Array<Events_MonthCreateManyInput>;

    @Field(() => Boolean, {nullable:true})
    skipDuplicates?: boolean;
}
