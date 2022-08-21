import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ControlPresetsWhereInput } from './control-presets-where.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteManyControlPresetsArgs {

    @Field(() => ControlPresetsWhereInput, {nullable:true})
    @Type(() => ControlPresetsWhereInput)
    where?: ControlPresetsWhereInput;
}
