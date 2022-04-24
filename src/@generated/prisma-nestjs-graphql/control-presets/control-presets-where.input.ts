import { Field } from '@nestjs/graphql';
import { InputType } from '@nestjs/graphql';
import { IntFilter } from '../prisma/int-filter.input';
import { StringFilter } from '../prisma/string-filter.input';

@InputType()
export class ControlPresetsWhereInput {

    @Field(() => [ControlPresetsWhereInput], {nullable:true})
    AND?: Array<ControlPresetsWhereInput>;

    @Field(() => [ControlPresetsWhereInput], {nullable:true})
    OR?: Array<ControlPresetsWhereInput>;

    @Field(() => [ControlPresetsWhereInput], {nullable:true})
    NOT?: Array<ControlPresetsWhereInput>;

    @Field(() => IntFilter, {nullable:true})
    MonitorId?: IntFilter;

    @Field(() => IntFilter, {nullable:true})
    Preset?: IntFilter;

    @Field(() => StringFilter, {nullable:true})
    Label?: StringFilter;
}
